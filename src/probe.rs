//! Strategy detection and module discovery.
//!
//! The public surface is four functions:
//!
//! * [`detect_strategies`] — run `probe.sh` against a single path and
//!   return the list of strategy names it prints (one per line).
//! * [`available_commands`] — list command names available for a given strategy.
//! * [`available_strategies_for_command`] — list strategy variants available for
//!   one command name.
//! * [`discover_modules`] — walk a directory tree and return every sub-directory
//!   that has at least one detected strategy and at least one runnable command.
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;
use walkdir::WalkDir;

use crate::config::MoldxConfig;

/// Synthetic strategy label used for strategy-agnostic commands.
pub const AGNOSTIC_STRATEGY: &str = "agnostic";

/// A module is a directory that [`probe.sh`] recognises as belonging to at
/// least one strategy.
#[derive(Debug, Clone)]
pub struct Module {
    /// Absolute path to the module directory.
    pub path: PathBuf,
    /// Map of strategy name → list of available command names.
    ///
    /// Strategy-agnostic commands are listed under [`AGNOSTIC_STRATEGY`].
    pub strategies: HashMap<String, Vec<String>>,
}

/// Hard ceiling on how long a single `probe.sh` invocation may take.
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Run `probe.sh` against `target` and return the strategy names it prints.
///
/// The script is invoked as `bash <probe_path> <target>` and is given
/// [`DETECTOR_TIMEOUT`] to complete. Each non-empty trimmed line of stdout
/// is treated as a strategy name. A non-zero exit code is treated as "no
/// strategies detected" rather than an error so that modules that simply do
/// not match any strategy are silently skipped during bulk discovery.
pub async fn detect_strategies(probe_path: &Path, target: &Path) -> Result<Vec<String>> {
    if !probe_path.exists() {
        anyhow::bail!("Probe script not found: {}", probe_path.display());
    }

    let output = timeout(
        PROBE_TIMEOUT,
        Command::new("bash").arg(probe_path).arg(target).output(),
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!(
            "probe.sh timed out after {}s",
            PROBE_TIMEOUT.as_secs()
        )
    })??;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Return the sorted list of command names available for `strategy`.
///
/// Layout supported:
/// - `.moldx/bin/<command>.sh` (strategy-agnostic)
/// - `.moldx/bin/<command>/<strategy>.sh` (strategy-specific)
pub fn available_commands(bin_dir: &Path, strategy: &str) -> Vec<String> {
    if strategy == AGNOSTIC_STRATEGY {
        return available_agnostic_commands(bin_dir);
    }

    let mut commands: Vec<String> = std::fs::read_dir(bin_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let command_dir = entry.path();
            if !command_dir.is_dir() {
                return None;
            }
            let command = command_dir.file_name()?.to_str()?.to_string();
            let variant = command_dir.join(format!("{}.sh", strategy));
            if variant.is_file() {
                Some(command)
            } else {
                None
            }
        })
        .collect();

    commands.sort();
    commands
}

/// Return the sorted strategy variants available for one command.
///
/// Only strategy-specific variants are returned. Strategy-agnostic availability
/// can be checked with [`has_agnostic_command`].
pub fn available_strategies_for_command(bin_dir: &Path, command: &str) -> Vec<String> {
    let command_dir = bin_dir.join(command);
    if !command_dir.is_dir() {
        return vec![];
    }

    let mut strategies: Vec<String> = std::fs::read_dir(command_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let p = entry.path();
            if p.extension().and_then(|x| x.to_str()) != Some("sh") {
                return None;
            }
            p.file_stem()?.to_str().map(String::from)
        })
        .collect();

    strategies.sort();
    strategies.dedup();
    strategies
}

/// Return true if `.moldx/bin/<command>.sh` exists.
pub fn has_agnostic_command(bin_dir: &Path, command: &str) -> bool {
    bin_dir.join(format!("{}.sh", command)).is_file()
}

fn available_agnostic_commands(bin_dir: &Path) -> Vec<String> {
    let mut commands: Vec<String> = std::fs::read_dir(bin_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let p = entry.path();
            if p.extension().and_then(|x| x.to_str()) == Some("sh") {
                p.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    commands.sort();
    commands
}

/// Walk `root` up to `max_depth` directory levels and return every directory
/// whose probe invocation emits at least one strategy name and that has at
/// least one runnable command for either a detected strategy or agnostic mode.
///
/// Directories whose names start with `.`, or equal `target` / `node_modules`,
/// are skipped entirely (including their subtrees). Results are sorted by path
/// for deterministic output.
pub async fn discover_modules(
    root: &Path,
    config: &MoldxConfig,
    max_depth: usize,
) -> Result<Vec<Module>> {
    let entries: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.') && name != "target" && name != "node_modules"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.into_path())
        .collect();

    let probe_path = Arc::new(config.probe_path.clone());
    let bin_dir = Arc::new(config.bin_dir.clone());
    // Limit to 8 concurrent probe.sh invocations
    let semaphore = Arc::new(Semaphore::new(8));
    let mut join_set = JoinSet::new();

    for path in entries {
        let probe_path = probe_path.clone();
        let bin_dir = bin_dir.clone();
        let semaphore = semaphore.clone();

        join_set.spawn(async move {
            // acquire() only fails if the semaphore is closed; we never close it
            let _permit = semaphore
                .acquire()
                .await
                .unwrap_or_else(|_| unreachable!("semaphore closed"));
            let detected = detect_strategies(&probe_path, &path)
                .await
                .unwrap_or_default();
            if detected.is_empty() {
                return None;
            }

            let mut strategy_map: HashMap<String, Vec<String>> = HashMap::new();

            let agnostic = available_commands(&bin_dir, AGNOSTIC_STRATEGY);
            if !agnostic.is_empty() {
                strategy_map.insert(AGNOSTIC_STRATEGY.to_string(), agnostic);
            }

            for strategy in detected {
                let commands = available_commands(&bin_dir, &strategy);
                if !commands.is_empty() {
                    strategy_map.insert(strategy, commands);
                }
            }

            if strategy_map.is_empty() {
                None
            } else {
                Some(Module {
                    path,
                    strategies: strategy_map,
                })
            }
        });
    }

    let mut modules = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok(Some(module)) = result {
            modules.push(module);
        }
    }
    modules.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── available_commands ────────────────────────────────────────────────────

    #[test]
    fn available_commands_returns_variant_matches_for_strategy() {
        let tmp = TempDir::new().unwrap();
        let build = tmp.path().join("build");
        let deploy = tmp.path().join("deploy");
        std::fs::create_dir(&build).unwrap();
        std::fs::create_dir(&deploy).unwrap();
        std::fs::write(build.join("docker.sh"), "").unwrap();
        std::fs::write(build.join("node.sh"), "").unwrap();
        std::fs::write(deploy.join("docker.sh"), "").unwrap();

        let cmds = available_commands(tmp.path(), "docker");
        assert_eq!(cmds, vec!["build", "deploy"]);
    }

    #[test]
    fn available_commands_returns_agnostic_root_scripts() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("diff.sh"), "").unwrap();
        std::fs::write(tmp.path().join("fmt.sh"), "").unwrap();
        std::fs::create_dir(tmp.path().join("build")).unwrap();

        let cmds = available_commands(tmp.path(), AGNOSTIC_STRATEGY);
        assert_eq!(cmds, vec!["diff", "fmt"]);
    }

    #[test]
    fn available_commands_empty_for_unknown_strategy() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("build")).unwrap();
        std::fs::write(tmp.path().join("build/docker.sh"), "").unwrap();

        let cmds = available_commands(tmp.path(), "rust");
        assert!(cmds.is_empty());
    }

    #[test]
    fn available_strategies_for_command_lists_sh_stems() {
        let tmp = TempDir::new().unwrap();
        let build = tmp.path().join("build");
        std::fs::create_dir(&build).unwrap();
        std::fs::write(build.join("docker.sh"), "").unwrap();
        std::fs::write(build.join("node.sh"), "").unwrap();
        std::fs::write(build.join("README.md"), "").unwrap();

        let variants = available_strategies_for_command(tmp.path(), "build");
        assert_eq!(variants, vec!["docker", "node"]);
    }

    #[test]
    fn has_agnostic_command_checks_root_script() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("diff.sh"), "").unwrap();
        assert!(has_agnostic_command(tmp.path(), "diff"));
        assert!(!has_agnostic_command(tmp.path(), "build"));
    }

    // ── detect_strategies ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn detect_strategies_parses_script_output() {
        let tmp = TempDir::new().unwrap();
        let probe = tmp.path().join("probe.sh");
        std::fs::write(&probe, "#!/usr/bin/env bash\necho alpha\necho beta\n").unwrap();

        let result = detect_strategies(&probe, tmp.path()).await.unwrap();
        assert_eq!(result, vec!["alpha", "beta"]);
    }

    #[tokio::test]
    async fn detect_strategies_returns_empty_on_no_output() {
        let tmp = TempDir::new().unwrap();
        let probe = tmp.path().join("probe.sh");
        std::fs::write(&probe, "#!/usr/bin/env bash\nexit 0\n").unwrap();

        let result = detect_strategies(&probe, tmp.path()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn detect_strategies_returns_empty_on_script_failure() {
        let tmp = TempDir::new().unwrap();
        let probe = tmp.path().join("probe.sh");
        std::fs::write(&probe, "#!/usr/bin/env bash\nexit 1\n").unwrap();

        let result = detect_strategies(&probe, tmp.path()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn detect_strategies_trims_whitespace() {
        let tmp = TempDir::new().unwrap();
        let probe = tmp.path().join("probe.sh");
        std::fs::write(&probe, "#!/usr/bin/env bash\necho '  docker  '\n").unwrap();

        let result = detect_strategies(&probe, tmp.path()).await.unwrap();
        assert_eq!(result, vec!["docker"]);
    }

    #[tokio::test]
    async fn detect_strategies_errors_when_no_probe() {
        let result = detect_strategies(
            std::path::Path::new("/nonexistent/probe.sh"),
            std::path::Path::new("/tmp"),
        )
        .await;
        assert!(result.is_err());
    }
}
