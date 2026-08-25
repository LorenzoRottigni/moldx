use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::PathBuf;

use crate::errors::MoldXError;
use crate::fs::{is_shell_script, validate_dir, validate_name};
use crate::types::Entity;
use anyhow::Result;

/// Represents an executable strategy script.
///
/// A `Command` wraps a single `.sh` file inside a strategy's bin directory.
/// The command name is derived from the file stem.
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub dir: PathBuf,
    pub format: String,
}

impl Command {
    /// Creates a new command from the given script path.
    ///
    /// # Arguments
    ///
    /// * `command_dir` - Path to the command script.
    ///
    /// # Returns
    ///
    /// A fully initialized [`Command`].
    ///
    /// # Errors
    ///
    /// Returns [`MoldXError::CommandNotFound`] if the path is not a `.sh`
    /// file, and [`MoldXError::InvalidName`] if the file stem cannot be
    /// determined or the derived name is not valid.
    pub fn new(command_dir: PathBuf) -> Result<Self> {
        if !command_dir.is_file() || !is_shell_script(&command_dir) {
            return Err(MoldXError::CommandNotFound {
                name: "".into(),
                path: command_dir,
            }
            .into());
        }

        let name = command_dir
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(MoldXError::InvalidName {
                entity: Entity::Command,
                name: command_dir.to_string_lossy().to_string(),
            })?
            .to_string();

        validate_name(name.clone(), Entity::Command)?;

        let format = command_dir
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            name,
            dir: command_dir,
            format,
        })
    }

    pub fn validate(&self) -> Result<bool> {
        validate_dir(&self.dir)
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{} {}",
            self.name.bold().cyan(),
            ".".dimmed(),
            self.format.yellow(),
            format!("@ {}", self.dir.display()).dimmed()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_command_new_valid() {
        let dir = tempdir().unwrap();
        let script = dir.path().join("build.sh");
        fs::write(&script, "#!/bin/bash\necho hi").unwrap();
        let cmd = Command::new(script.clone()).unwrap();
        assert_eq!(cmd.name, "build");
        assert_eq!(cmd.format, "sh");
        assert_eq!(cmd.dir, script);
    }

    #[test]
    fn test_command_new_not_a_file() {
        let dir = tempdir().unwrap();
        let result = Command::new(dir.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn test_command_new_not_shell_script() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("build.py");
        fs::write(&file, "#!/usr/bin/env python3").unwrap();
        let result = Command::new(file);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_new_invalid_name_via_validate() {
        let result = validate_name("..".into(), Entity::Command);
        assert!(result.is_err());
        let result = validate_name(".".into(), Entity::Command);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_display() {
        let dir = tempdir().unwrap();
        let script = dir.path().join("test.sh");
        fs::write(&script, "").unwrap();
        let cmd = Command::new(script).unwrap();
        let display = cmd.to_string();
        assert!(display.contains("test"));
        assert!(display.contains("sh"));
    }
}
