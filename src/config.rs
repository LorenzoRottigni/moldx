use crate::constants::MOLDX_DIR_NAME;
use crate::errors::MoldXError;
use crate::fs;
use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MoldXConfig {
    pub moldx_dir: PathBuf,
    pub strategies_dir: PathBuf,
    pub bin_dir_name: String,
    pub template_dir_name: String,
    pub templates_dir_name: String,
    pub modules_dir: PathBuf,
    pub max_resolution_depth: usize,
}

impl MoldXConfig {
    pub fn new(
        moldx_dir: String,
        strategies_dir_name: String,
        bin_dir_name: String,
        template_dir_name: String,
        templates_dir_name: String,
        max_resolution_depth: usize,
        modules_dir: Option<String>,
    ) -> Result<Self, MoldXError> {
        // if not provided must be determined (if default doesnt exist)
        let mut moldx_dir = PathBuf::from(moldx_dir);
        if !moldx_dir.exists() {
            moldx_dir = fs::discover_path(
                std::env::current_dir().map_err(|_| MoldXError::CurrentDir)?,
                |path| path.file_name().and_then(|n| n.to_str()) == Some(MOLDX_DIR_NAME),
                max_resolution_depth,
                true,
            )?;
        }
        // if not provided is the parent of moldx_dir
        let modules_dir = if let Some(dir) = modules_dir {
            PathBuf::from(&dir)
        } else {
            moldx_dir
                .parent()
                .ok_or_else(|| MoldXError::RunUsage)?
                .to_path_buf()
        };

        Ok(Self {
            strategies_dir: moldx_dir.join(strategies_dir_name),
            moldx_dir,
            bin_dir_name,
            template_dir_name,
            templates_dir_name,
            modules_dir,
            max_resolution_depth,
        })
    }
}

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
            "strategies_dir:".bold().blue(),
            self.strategies_dir.display()
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
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_config_new_with_existing_dir() {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        fs::create_dir(&moldx_dir).unwrap();
        let config = MoldXConfig::new(
            moldx_dir.to_str().unwrap().into(),
            "strategies".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            None,
        ).unwrap();
        assert_eq!(config.moldx_dir, moldx_dir);
        assert_eq!(config.strategies_dir, moldx_dir.join("strategies"));
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
            "strategies".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            Some(modules_dir.to_str().unwrap().into()),
        ).unwrap();
        assert_eq!(config.modules_dir, modules_dir);
    }

    #[test]
    fn test_config_display() {
        let dir = tempdir().unwrap();
        let moldx_dir = dir.path().join(".moldx");
        fs::create_dir(&moldx_dir).unwrap();
        let config = MoldXConfig::new(
            moldx_dir.to_str().unwrap().into(),
            "strategies".into(),
            "bin".into(),
            "template".into(),
            "templates".into(),
            20,
            None,
        ).unwrap();
        let display = config.to_string();
        assert!(display.contains("moldx_dir:"));
        assert!(display.contains("strategies_dir:"));
        assert!(display.contains("modules_dir:"));
    }
}
