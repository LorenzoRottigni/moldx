//! Configuration resolution for a moldx project.
//!
//! The core concept is simple: a project that uses moldx has a `.moldx/`
//! directory somewhere in its tree.  [`MoldxConfig::resolve`] finds it by
//! walking *up* from the given starting path, exactly like `git` locates
//! `.git/`.
//!
//! ## Directory layout
//!
//! ```text
//! <project-root>/
//!   .moldx/
//!     detector.sh          # receives a module path, prints strategy names
//!     commands/
//!       <command>.sh                 # strategy-agnostic command
//!       <command>/<strategy>.sh      # strategy-specific variant
//! ```
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// Resolved locations of all moldx configuration artefacts.
///
/// Cloning is cheap — all fields are [`PathBuf`]s.
#[derive(Clone)]
pub struct MoldxConfig {
    /// Project root (directory containing `.moldx/`)
    pub root: PathBuf,
    /// `.moldx/` directory — kept for callers that need to locate the config root
    #[allow(dead_code)]
    pub moldx_dir: PathBuf,
    /// `.moldx/detector.sh`
    pub detector_path: PathBuf,
    /// `.moldx/commands/`
    pub commands_dir: PathBuf,
    /// `.moldx/.state.json` — cross-session process registry
    pub state_file_path: PathBuf,
}

impl MoldxConfig {
    /// Resolve configuration starting from `start`.
    ///
    /// Precedence (highest first):
    /// 1. `moldx_dir_override` / `MOLDX_DIR` env var
    /// 2. `commands_dir_override` / `MOLDX_COMMANDS_DIR` env var (only
    ///    affects the commands directory, not the detector)
    /// 3. Auto-discovery: walk up from `start` until a `.moldx/` directory is found
    pub fn resolve(
        start: &Path,
        moldx_dir_override: Option<&Path>,
        commands_dir_override: Option<&Path>,
    ) -> Result<Self> {
        let moldx_dir = if let Some(p) = moldx_dir_override {
            p.to_path_buf()
        } else {
            find_moldx_dir(start)?
        };

        let root = moldx_dir.parent().unwrap_or(&moldx_dir).to_path_buf();

        let commands_dir = commands_dir_override
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| moldx_dir.join("commands"));

        let detector_path = moldx_dir.join("detector.sh");
        let state_file_path = moldx_dir.join(".state.json");

        Ok(MoldxConfig {
            root,
            moldx_dir,
            detector_path,
            commands_dir,
            state_file_path,
        })
    }
}

/// Walk upward from `start` until a directory named `.moldx` is found.
///
/// Returns an error if the filesystem root is reached without finding one.
fn find_moldx_dir(start: &Path) -> Result<PathBuf> {
    let mut current = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().unwrap_or(start).to_path_buf()
    };

    loop {
        let candidate = current.join(".moldx");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => bail!(
                "No .moldx directory found (searched up from {}).\n\
                 Create ./.moldx/detector.sh and ./.moldx/commands/ to get started.",
                start.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn finds_moldx_dir_in_start_directory() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".moldx")).unwrap();
        let result = find_moldx_dir(tmp.path()).unwrap();
        assert_eq!(result, tmp.path().join(".moldx"));
    }

    #[test]
    fn finds_moldx_dir_in_ancestor_directory() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".moldx")).unwrap();
        let deep = tmp.path().join("a/b/c");
        std::fs::create_dir_all(&deep).unwrap();
        let result = find_moldx_dir(&deep).unwrap();
        assert_eq!(result, tmp.path().join(".moldx"));
    }

    #[test]
    fn errors_when_no_moldx_dir_exists() {
        let tmp = TempDir::new().unwrap();
        // TempDir has no .moldx; walk will reach filesystem root and fail
        let result = find_moldx_dir(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".moldx"));
    }

    #[test]
    fn resolve_uses_override_when_provided() {
        let tmp = TempDir::new().unwrap();
        let moldx = tmp.path().join(".moldx");
        std::fs::create_dir(&moldx).unwrap();
        let cfg =
            MoldxConfig::resolve(std::path::Path::new("/nonexistent"), Some(&moldx), None).unwrap();
        assert_eq!(cfg.moldx_dir, moldx);
        assert_eq!(cfg.detector_path, moldx.join("detector.sh"));
        assert_eq!(cfg.commands_dir, moldx.join("commands"));
        assert_eq!(cfg.state_file_path, moldx.join(".state.json"));
    }

    #[test]
    fn resolve_uses_custom_commands_dir_when_provided() {
        let tmp = TempDir::new().unwrap();
        let moldx = tmp.path().join(".moldx");
        let commands = tmp.path().join("my-commands");
        std::fs::create_dir(&moldx).unwrap();
        let cfg = MoldxConfig::resolve(tmp.path(), Some(&moldx), Some(&commands)).unwrap();
        assert_eq!(cfg.commands_dir, commands);
    }
}
