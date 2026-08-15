//! Configuration resolution for a moldx project.
//!
//! A moldx-enabled project contains a `.moldx/` directory somewhere in its
//! tree. [`MoldxConfig::resolve`] finds that directory by walking upward from
//! the starting path, then exposes the resolved strategies directory.
//!
//! ## Directory layout
//!
//! ```text
//! <project-root>/
//!   .moldx/
//!     strategies/
//!       docker/
//!         bin/
//!           build.sh
//!           deploy.sh
//!         template/
//!           Dockerfile
//!       default/
//!         template/        # empty => agnostic commands
//!         bin/
//!           diff.sh
//! ```
use anyhow::{bail, Result};
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
    /// `.moldx/strategies/`
    pub strategies_dir: PathBuf,
}

impl MoldxConfig {
    /// Resolve configuration starting from `start`.
    ///
    /// Precedence (highest first):
    /// 1. `moldx_dir_override` / `MOLDX_DIR` env var
    /// 2. `strategies_dir_override` / `MOLDX_STRATEGIES_DIR` env var
    /// 3. Auto-discovery: walk up from `start` until a `.moldx/` directory is found
    pub fn resolve(
        start: &Path,
        moldx_dir_override: Option<&Path>,
        strategies_dir_override: Option<&Path>,
    ) -> Result<Self> {
        let moldx_dir = if let Some(p) = moldx_dir_override {
            p.to_path_buf()
        } else {
            find_moldx_dir(start)?
        };

        let root = moldx_dir.parent().unwrap_or(&moldx_dir).to_path_buf();

        let strategies_dir = strategies_dir_override
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| moldx_dir.join("strategies"));

        Ok(MoldxConfig {
            root,
            moldx_dir,
            strategies_dir,
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
                 Create ./.moldx/strategies/ to get started.",
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
        assert_eq!(cfg.strategies_dir, moldx.join("strategies"));
    }

    #[test]
    fn resolve_uses_custom_strategies_dir_when_provided() {
        let tmp = TempDir::new().unwrap();
        let moldx = tmp.path().join(".moldx");
        let strategies = tmp.path().join("my-strategies");
        std::fs::create_dir(&moldx).unwrap();
        let cfg = MoldxConfig::resolve(tmp.path(), Some(&moldx), Some(&strategies)).unwrap();
        assert_eq!(cfg.strategies_dir, strategies);
    }
}
