//! `moldx run` subcommand.
//!
//! Parses raw arguments and executes profile commands against one or more
//! modules, supporting glob patterns and forwarded command options.

use anyhow::Result;
use dialoguer::Select;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::{client::MoldXClient, errors::MoldXError2};

/// Splits raw run arguments into profiles, command, module targets, and
/// command options.
///
/// The CLI grammar is `moldx [PROFILE...] <COMMAND> [MODULE...] [-- <OPTIONS>...]`.
/// Profile names are recognized from the left; the first non-profile token is
/// the command; everything after it (up to `--`) is a module path or glob
/// pattern; everything after `--` is forwarded unchanged to the command.
#[derive(Debug)]
struct RunArgs {
    profiles: Vec<String>,
    command: String,
    modules: Vec<String>,
    options: Vec<String>,
}

impl RunArgs {
    fn parse(client: &MoldXClient, args: Vec<String>) -> Result<Self> {
        let mut split_on_dash = args.split(|a| a == "--");
        let mut before = split_on_dash.next().map(|v| v.to_vec()).unwrap_or_default();
        let options = split_on_dash.next().map(|v| v.to_vec()).unwrap_or_default();

        // Remove the `--` marker itself if it leaked into `before`.
        before.retain(|a| a != "--");

        let profile_names = collect_profile_names(client);

        let mut profiles = Vec::new();
        let mut command = None;
        let mut modules = Vec::new();

        for token in before {
            match command {
                None if profile_names.contains(&token) => profiles.push(token),
                None => {
                    command = Some(token);
                }
                Some(_) => modules.push(token),
            }
        }

        let command = command.ok_or(MoldXError2::RunUsage)?;

        Ok(Self {
            profiles,
            command,
            modules,
            options,
        })
    }
}

/// Collects every profile name in the client, including nested profiles.
fn collect_profile_names(client: &MoldXClient) -> Vec<String> {
    fn walk(profiles: &[crate::profile::Profile], out: &mut Vec<String>) {
        for p in profiles {
            out.push(p.name.clone());
            walk(&p.profiles, out);
        }
    }
    let mut names = Vec::new();
    if let Some(root) = client.profiles.first() {
        walk(&root.profiles, &mut names);
    }
    names
}

/// Expands a module spec into concrete module paths.
///
/// Literal paths are returned as-is. Glob patterns containing `*` or `**` are
/// matched against the client's resolved modules relative to the current
/// working directory.
fn expand_modules(client: &MoldXClient, spec: &str) -> Vec<PathBuf> {
    let pattern = Path::new(spec);
    let pattern_str = spec;

    // No glob chars -> literal path.
    if !pattern_str.contains('*') {
        return vec![pattern.to_path_buf()];
    }

    let pattern_segments: Vec<&str> = pattern_str.split('/').filter(|s| !s.is_empty()).collect();

    // A pattern like `packages/**` should also match the top-level module.
    let mut matched = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for module in &client.modules {
        let rel = match module.path.strip_prefix(&cwd) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => continue,
        };
        let rel_str = rel.to_string_lossy().into_owned();
        let segments: Vec<&str> = rel_str.split('/').filter(|s| !s.is_empty()).collect();
        if glob_match(&pattern_segments, &segments) {
            matched.push(module.path.clone());
        }
    }
    matched.sort();
    matched.dedup();
    matched
}

/// Matches candidate path segments against a glob pattern supporting `*`
/// (single segment) and `**` (zero or more segments).
fn glob_match(pattern: &[&str], candidate: &[&str]) -> bool {
    fn rec(p: &[&str], c: &[&str]) -> bool {
        match (p.first(), c.first()) {
            (None, None) | (Some(&"**"), _) if p.len() == 1 => {
                // trailing `**` matches any remainder (including none)
                true
            }
            (None, _) => c.is_empty(),
            (Some(&"**"), _) => {
                // `**` can match zero or more candidate segments
                rec(&p[1..], c) || (!c.is_empty() && rec(p, &c[1..]))
            }
            (Some(_), None) => false,
            (Some(&"*"), Some(_)) => {
                if p.len() == 1 {
                    c.len() == 1
                } else {
                    rec(&p[1..], &c[1..])
                }
            }
            (Some(seg), Some(cand)) => {
                if *seg == *cand {
                    rec(&p[1..], &c[1..])
                } else {
                    false
                }
            }
        }
    }
    rec(pattern, candidate)
}

/// Runs a profile command for one or more modules.
///
/// Accepts `<command> <path>`, `<profile> <command> <path>`, or a nested
/// profile hierarchy such as `python uv build <path>`. Glob patterns (`*` for
/// a single level, `**` for recursion) target multiple modules, resolving the
/// command independently for each. Arguments following `--` are forwarded to
/// the resolved command unchanged.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw run arguments.
/// * `skip_conflicts` - Auto-select the first matching command on conflict.
///
/// # Returns
///
/// Ok when every command completes with exit code zero.
///
/// # Errors
///
/// Returns an error when the path does not exist, a hinted profile does not
/// apply, a command is unknown, or script execution fails.
pub async fn run(client: &MoldXClient, args: Vec<String>, skip_conflicts: bool) -> Result<()> {
    let args = RunArgs::parse(client, args)?;

    let mut targets: Vec<PathBuf> = Vec::new();
    for spec in &args.modules {
        targets.extend(expand_modules(client, spec));
    }
    targets.sort();
    targets.dedup();

    if targets.is_empty() {
        if let Some(command) = client
            .profiles
            .first()
            .and_then(|root| root.get_local_command(&args.command))
        {
            let code = client
                .executor
                .exec_blocking_optional(None, &command.path, &args.options)
                .await?;
            if code != 0 {
                return Err(MoldXError2::ProcessNonZeroExit { code }.into());
            }
            return Ok(());
        }

        // Profile commands without an explicit module retain the convenient
        // current-directory behavior.
        let cwd = std::env::current_dir().map_err(|_| MoldXError2::CwdNotFound)?;
        return run_module(
            client,
            &args.command,
            &cwd,
            &args.profiles,
            &args.options,
            skip_conflicts,
        )
        .await;
    }

    for module in &targets {
        run_module(
            client,
            &args.command,
            module,
            &args.profiles,
            &args.options,
            skip_conflicts,
        )
        .await?;
    }

    Ok(())
}

/// Runs a single command against one module with conflict resolution.
///
/// A matching command is resolved with [`MoldXClient::commands_for_module`].
/// When several profiles expose the command, the caller chooses via
/// `--skip-conflicts`, an interactive prompt, or an error if no TTY is
/// available.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `command_name` - Name of the command to execute.
/// * `module_path` - The module the command runs against.
/// * `profile_names` - Optional profile hierarchy restricting resolution.
/// * `options` - Arguments forwarded to the command after the module path.
/// * `skip_conflicts` - Auto-select the first command when several match.
///
/// # Returns
///
/// Ok when the command exits with status zero.
///
/// # Errors
///
/// Returns an error if the module path does not exist, no command can be
/// resolved, or script execution fails.
async fn run_module(
    client: &MoldXClient,
    command_name: &str,
    module_path: &Path,
    profile_names: &[String],
    options: &[String],
    skip_conflicts: bool,
) -> Result<()> {
    if !module_path.exists() {
        return Err(MoldXError2::PathNotFound {
            path: module_path.to_path_buf(),
            kind: "module",
        }
        .into());
    }

    let commands = client.commands_for_module(command_name, module_path, profile_names);

    if commands.is_empty() {
        if !profile_names.is_empty() {
            let requested = profile_names[0].clone();
            let available = client.profiles_for_module(module_path);
            if !available.iter().any(|profile| profile.name == requested) {
                return Err(MoldXError2::ProfileNotAvailable {
                    name: requested,
                    path: module_path.to_path_buf(),
                }
                .into());
            }
            return Err(MoldXError2::CommandNotFoundInProfile {
                name: command_name.to_string(),
                profile: requested,
            }
            .into());
        }
        return Err(MoldXError2::CommandNotFound {
            name: command_name.to_string(),
            path: module_path.to_path_buf(),
        }
        .into());
    }

    let command = if commands.len() > 1 {
        if skip_conflicts {
            &commands[0]
        } else if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
            let selection = Select::new()
                .with_prompt("Multiple commands found, select one")
                .items(&commands)
                .default(0)
                .interact()?;
            &commands[selection]
        } else {
            return Err(anyhow::anyhow!(
                "Multiple commands found for '{}' in module '{}'; rerun with --skip-conflicts or a TTY",
                command_name,
                module_path.display()
            ));
        }
    } else {
        &commands[0]
    };

    let code = client
        .executor
        .exec_blocking(&command.path, module_path, options)
        .await?;

    if code != 0 {
        return Err(MoldXError2::ProcessNonZeroExit { code }.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_glob_match_literal() {
        assert!(glob_match(&["packages", "api"], &["packages", "api"]));
        assert!(!glob_match(&["packages", "api"], &["packages", "worker"]));
    }

    #[test]
    fn test_glob_match_single_star() {
        assert!(glob_match(&["packages", "*"], &["packages", "api"]));
        assert!(glob_match(&["packages", "*"], &["packages", "worker"]));
        assert!(!glob_match(
            &["packages", "*"],
            &["packages", "sub", "deep"]
        ));
    }

    #[test]
    fn test_glob_match_double_star() {
        assert!(glob_match(&["packages", "**"], &["packages", "api"]));
        assert!(glob_match(
            &["packages", "**"],
            &["packages", "sub", "deep"]
        ));
        assert!(!glob_match(&["packages", "**"], &["other", "api"]));
    }

    #[test]
    fn test_glob_match_double_star_mid() {
        assert!(glob_match(&["a", "**", "c"], &["a", "b", "c"]));
        assert!(glob_match(&["a", "**", "c"], &["a", "c"]));
        assert!(glob_match(&["a", "**", "c"], &["a", "x", "y", "c"]));
        assert!(!glob_match(&["a", "**", "c"], &["a", "b", "d"]));
    }

    #[test]
    fn test_glob_match_trailing_double_star_matches_zero() {
        assert!(glob_match(&["packages", "**"], &["packages"]));
    }

    fn empty_client() -> MoldXClient {
        let dir = tempfile::tempdir().unwrap();

        fn mk_profile(base: &std::path::Path, name: &str) {
            let p = base.join(name);
            fs::create_dir_all(p.join("bin")).unwrap();
            fs::create_dir_all(p.join("template")).unwrap();
            fs::write(p.join("template").join("Dockerfile"), "").unwrap();
        }

        let profiles = dir.path().join(".moldx/profiles");
        fs::create_dir_all(&profiles).unwrap();
        mk_profile(&profiles, "docker");
        mk_profile(&profiles, "node");
        let py = profiles.join("python");
        fs::create_dir_all(py.join("bin")).unwrap();
        fs::create_dir_all(py.join("template")).unwrap();
        let uv = py.join("profiles").join("uv");
        fs::create_dir_all(uv.join("bin")).unwrap();
        fs::create_dir_all(uv.join("template")).unwrap();
        fs::write(uv.join("template").join("pyproject.toml"), "").unwrap();

        let config = crate::config::MoldXConfig {
            moldx_dir: dir.path().join(".moldx"),
            profiles_dir: profiles.clone(),
            profiles_dir_name: "profiles".into(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: dir.path().to_path_buf(),
            max_resolution_depth: 20,
        };
        MoldXClient::new(config).unwrap()
    }

    #[test]
    fn test_parse_profiles_command_module() {
        let client = empty_client();
        let args = RunArgs::parse(
            &client,
            vec!["docker".into(), "build".into(), "packages/api".into()],
        )
        .unwrap();
        assert_eq!(args.profiles, vec!["docker"]);
        assert_eq!(args.command, "build");
        assert_eq!(args.modules, vec!["packages/api"]);
        assert!(args.options.is_empty());
    }

    #[test]
    fn test_parse_nested_profiles_command_module() {
        let client = empty_client();
        let args = RunArgs::parse(
            &client,
            vec![
                "python".into(),
                "uv".into(),
                "build".into(),
                "packages/worker".into(),
            ],
        )
        .unwrap();
        assert_eq!(args.profiles, vec!["python", "uv"]);
        assert_eq!(args.command, "build");
        assert_eq!(args.modules, vec!["packages/worker"]);
    }

    #[test]
    fn test_parse_no_profile() {
        let client = empty_client();
        let args = RunArgs::parse(&client, vec!["build".into(), "packages/api".into()]).unwrap();
        assert!(args.profiles.is_empty());
        assert_eq!(args.command, "build");
        assert_eq!(args.modules, vec!["packages/api"]);
    }

    #[test]
    fn test_parse_command_options_after_double_dash() {
        let client = empty_client();
        let args = RunArgs::parse(
            &client,
            vec![
                "docker".into(),
                "build".into(),
                "packages/api".into(),
                "--".into(),
                "--platform".into(),
                "linux/amd64".into(),
                "--push".into(),
            ],
        )
        .unwrap();
        assert_eq!(args.command, "build");
        assert_eq!(args.modules, vec!["packages/api"]);
        assert_eq!(args.options, vec!["--platform", "linux/amd64", "--push"]);
    }

    #[test]
    fn test_parse_multiple_modules() {
        let client = empty_client();
        let args = RunArgs::parse(
            &client,
            vec!["install".into(), "packages/a".into(), "packages/b".into()],
        )
        .unwrap();
        assert_eq!(args.command, "install");
        assert_eq!(args.modules, vec!["packages/a", "packages/b"]);
    }

    #[test]
    fn test_parse_command_without_module() {
        let client = empty_client();
        let args = RunArgs::parse(&client, vec!["docker".into(), "build".into()]).unwrap();
        assert_eq!(args.profiles, vec!["docker"]);
        assert_eq!(args.command, "build");
        assert!(args.modules.is_empty());
    }

    #[test]
    fn test_parse_no_args_errors() {
        let client = empty_client();
        let result = RunArgs::parse(&client, vec![]);
        assert!(result.is_err());
    }
}
