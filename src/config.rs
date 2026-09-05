use crate::constants::MOLDX_DIR_NAME;
use crate::errors::MoldXError2;
use crate::fs;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::PathBuf;

/// Configuration for a MoldX project.
///
/// Holds all directory paths and naming conventions needed to locate and
/// resolve profiles, modules, and templates within a `.moldx` tree.
#[derive(Debug, Clone)]
pub struct MoldXConfig {
    pub moldx_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub profiles_dir_name: String,
    pub bin_dir_name: String,
    pub template_dir_name: String,
    pub templates_dir_name: String,
    pub modules_dir: PathBuf,
    pub max_resolution_depth: usize,
}

impl MoldXConfig {
    /// Builds a new configuration, discovering the `.moldx` directory if it
    /// does not exist at the given path.
    ///
    /// When `moldx_dir` does not exist, the filesystem is searched upward
    /// from the current working directory for a `.moldx` directory. The
    /// modules directory defaults to the parent of the resolved `.moldx`
    /// directory unless an explicit path is provided.
    ///
    /// # Arguments
    ///
    /// * `moldx_dir` - Candidate path to the `.moldx` directory.
    /// * `profiles_dir_name` - Name of the profiles subdirectory.
    /// * `bin_dir_name` - Name of the bin subdirectory inside profiles.
    /// * `template_dir_name` - Name of the template subdirectory inside profiles.
    /// * `templates_dir_name` - Name of the templates subdirectory inside profiles.
    /// * `max_resolution_depth` - Maximum upward search depth.
    /// * `modules_dir` - Optional explicit modules root.
    /// * `create_if_missing` - When true and the `.moldx` directory is absent,
    ///   use the given path as-is instead of requiring filesystem discovery.
    ///   This supports scaffolding commands (e.g. `init`) that create `.moldx`.
    ///
    /// # Returns
    ///
    /// The resolved configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the `.moldx` directory cannot be found or the
    /// modules root cannot be determined.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        moldx_dir: String,
        profiles_dir_name: String,
        bin_dir_name: String,
        template_dir_name: String,
        templates_dir_name: String,
        max_resolution_depth: usize,
        modules_dir: Option<String>,
        create_if_missing: bool,
    ) -> Result<Self> {
        let mut moldx_dir = PathBuf::from(moldx_dir);
        if !moldx_dir.exists() && !create_if_missing {
            moldx_dir = fs::discover_path(
                std::env::current_dir().map_err(|_| MoldXError2::CwdNotFound)?,
                |path| path.file_name().and_then(|n| n.to_str()) == Some(MOLDX_DIR_NAME),
                max_resolution_depth,
                true,
            )?;
        }
        // Canonicalize an existing relative path (e.g. `./.moldx`) so that the
        // modules root derived from its parent is absolute and walkable.
        if moldx_dir.exists() {
            moldx_dir = moldx_dir.canonicalize()?;
        }
        let modules_dir = if let Some(dir) = modules_dir {
            PathBuf::from(&dir)
        } else {
            moldx_dir
                .parent()
                .ok_or_else(|| MoldXError2::ModulesRootResolutionFailed {
                    path: moldx_dir.clone(),
                })?
                .to_path_buf()
        };
        let profiles_dir_name_clone = profiles_dir_name.clone();
        let profiles_dir = moldx_dir.join(profiles_dir_name);
        Ok(Self {
            profiles_dir,
            profiles_dir_name: profiles_dir_name_clone,
            moldx_dir,
            bin_dir_name,
            template_dir_name,
            templates_dir_name,
            modules_dir,
            max_resolution_depth,
        })
    }
}

/// Prints the three key directory paths.
impl Display for MoldXConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {}",
            "moldx_dir:".bold().blue(),
            self.moldx_dir.display()
        )?;
        writeln!(
            f,
            "{} {}",
            "profiles_dir:".bold().blue(),
            self.profiles_dir.display()
        )?;
        writeln!(
            f,
            "{} {}",
            "modules_dir:".bold().blue(),
            self.modules_dir.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_new_with_existing_dir() {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        fs::create_dir(&moldx_dir).unwrap();
        let config = MoldXConfig::new(
            moldx_dir.to_str().unwrap().into(),
            "profiles".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            None,
            false,
        )
        .unwrap();
        assert_eq!(config.moldx_dir, moldx_dir);
        assert_eq!(config.profiles_dir, moldx_dir.join("profiles"));
        assert_eq!(config.modules_dir, dir.path());
        assert_eq!(config.bin_dir_name, "bin");
        assert_eq!(config.template_dir_name, "template");
        assert_eq!(config.templates_dir_name, "templates");
        assert_eq!(config.max_resolution_depth, 20);
    }

    #[test]
    fn test_config_new_with_explicit_modules_dir() {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        fs::create_dir(&moldx_dir).unwrap();
        let modules_dir = dir.path().join("custom_modules");
        fs::create_dir(&modules_dir).unwrap();
        let config = MoldXConfig::new(
            moldx_dir.to_str().unwrap().into(),
            "profiles".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            Some(modules_dir.to_str().unwrap().into()),
            false,
        )
        .unwrap();
        assert_eq!(config.modules_dir, modules_dir);
    }

    #[test]
    fn test_config_display() {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        fs::create_dir(&moldx_dir).unwrap();
        let config = MoldXConfig::new(
            moldx_dir.to_str().unwrap().into(),
            "profiles".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            None,
            false,
        )
        .unwrap();
        let display = config.to_string();
        assert!(display.contains("moldx_dir:"));
        assert!(display.contains("profiles_dir:"));
        assert!(display.contains("modules_dir:"));
    }

    #[test]
    fn test_config_new_with_create_if_missing_uses_literal_path() {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        // Directory intentionally does not exist.
        let config = MoldXConfig::new(
            moldx_dir.to_str().unwrap().into(),
            "profiles".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            None,
            true,
        )
        .unwrap();
        assert_eq!(config.moldx_dir, moldx_dir);
        assert_eq!(config.profiles_dir, moldx_dir.join("profiles"));
        assert_eq!(config.modules_dir, dir.path());
    }
}
