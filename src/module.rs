use std::path::PathBuf;
use std::fmt::{self, Display};
use owo_colors::OwoColorize;
use anyhow::Result;

use crate::errors::MoldXError;
use crate::fs::validate_name;
use crate::types::Entity;

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub dir: PathBuf,
    pub strategies: Vec<usize>,
}

impl Module {
    pub fn new(dir: PathBuf, strategies: Vec<usize>) -> Result<Self> {
        let name = dir
            .file_name()
            .ok_or_else(|| MoldXError::ModuleDirNoFileName { path: dir.clone() })?
            .to_string_lossy()
            .into_owned();
        validate_name(name.clone(), Entity::Module)?;
        Ok(Self {
            name,
            dir,
            strategies
        })
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strategy_count = self.strategies.len();
        let strategy_label = match strategy_count {
            0 => "no strategies".dimmed().to_string(),
            1 => "1 strategy".cyan().to_string(),
            n => format!("{} strategies", n).cyan().to_string(),
        };

        write!(
            f,
            "{} [{}] {}",
            self.name.bold().green(),
            strategy_label,
            format!("@ {}", self.dir.display()).dimmed()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_module_new_valid() {
        let dir = tempdir().unwrap();
        let module_dir = dir.path().join("my-module");
        fs::create_dir(&module_dir).unwrap();
        let m = Module::new(module_dir.clone(), vec![0, 2]).unwrap();
        assert_eq!(m.name, "my-module");
        assert_eq!(m.dir, module_dir);
        assert_eq!(m.strategies, vec![0, 2]);
    }

    #[test]
    fn test_module_new_no_filename() {
        let result = Module::new(PathBuf::from("/"), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_module_new_invalid_name_via_validate() {
        let result = validate_name("..".into(), Entity::Module);
        assert!(result.is_err());
        let result = validate_name(".".into(), Entity::Module);
        assert!(result.is_err());
    }

    #[test]
    fn test_module_display_zero_strategies() {
        let dir = tempdir().unwrap();
        let m = Module::new(dir.path().to_path_buf(), vec![]).unwrap();
        let display = m.to_string();
        assert!(display.contains("no strategies"));
    }

    #[test]
    fn test_module_display_one_strategy() {
        let dir = tempdir().unwrap();
        let m = Module::new(dir.path().to_path_buf(), vec![0]).unwrap();
        let display = m.to_string();
        assert!(display.contains("1 strategy"));
    }

    #[test]
    fn test_module_display_multiple_strategies() {
        let dir = tempdir().unwrap();
        let m = Module::new(dir.path().to_path_buf(), vec![0, 1, 2]).unwrap();
        let display = m.to_string();
        assert!(display.contains("3 strategies"));
    }
}
