use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::PathBuf;

use crate::errors::{MoldXError, MoldXError2};
use crate::fs::{is_shell_script, resolve_name, validate_dir, validate_name};
use crate::types::Entity;
use anyhow::{Result, bail};

/// Represents an executable strategy script.
///
/// A `Command` wraps a single `.sh` file inside a strategy's bin directory.
/// The command name is derived from the file stem.
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub path: PathBuf,
    pub format: String,
}

impl Command {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path,
                kind: "template",
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
}

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
