use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use crate::errors::{MoldXError2};
use crate::fs::{is_shell_script, resolve_name, sorted_read_dir};
use crate::types::Entity;
use anyhow::{Result, bail};

/// Represents an executable profile script.
///
/// A `Command` wraps a single `.sh` file inside a profile's bin directory.
/// The command name is derived from the file stem.
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub path: PathBuf,
    pub format: String,
}

impl Command {
    /// Loads a command from a shell script file.
    ///
    /// The command name is derived from the file stem and the format from
    /// the file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.sh` script.
    ///
    /// # Returns
    ///
    /// The loaded command.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a file, is not a shell script,
    /// or its name cannot be resolved.
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.is_file() {
            bail!(MoldXError2::PathNotFound {
                path,
                kind: "command",
            });
        }

        if !is_shell_script(&path) {
            bail!(MoldXError2::InvalidCommandFormat { path })
        }

        let name = resolve_name(&path, Entity::Command)?;

        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            name,
            path,
            format,
        })
    }

    /// Discovers and loads all commands in a directory.
    ///
    /// Only files are considered; subdirectories are ignored.
    ///
    /// # Arguments
    ///
    /// * `source` - The directory to scan for command scripts.
    ///
    /// # Returns
    ///
    /// The list of resolved commands, sorted by file name.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory does not exist or a script cannot
    /// be loaded.
    pub fn resolve_commands(source: &Path) -> Result<Vec<Command>> {
        if !source.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: source.to_path_buf(),
                kind: "profile bin",
            });
        }

        sorted_read_dir(source)?
            .into_iter()
            .filter(|e| e.path().is_file() && is_shell_script(&e.path()))
            .map(|e| Command::new(e.path()))
            .collect()
    }
}

/// Prints the command name, file extension, and path.
impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{} {}",
            self.name.bold().cyan(),
            ".".dimmed(),
            self.format.yellow(),
            format!("@ {}", self.path.display()).dimmed()
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
        assert_eq!(cmd.path, script);
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
    fn test_command_new_invalid_name() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("..");
        let result = Command::new(bad_path);
        assert!(result.is_err());
        let bad_path = dir.path().join(".");
        let result = Command::new(bad_path);
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
