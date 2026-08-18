use std::path::PathBuf;
use std::fmt::{self, Display};
use owo_colors::OwoColorize;

use crate::errors::MoldXError;
use crate::fs::{is_shell_script, validate_name};
use crate::types::Entity;
use anyhow:: Result;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub dir: PathBuf,
    pub format: String,
}

impl Command {
    pub fn new(command_dir: PathBuf) -> Result<Self> {
        if !command_dir.is_file() || !is_shell_script(&command_dir) {
            return Err(MoldXError::CommandNotFound { name: "".into(), path: command_dir }.into())
        }

        let name = command_dir
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(MoldXError::InvalidName { entity: Entity::Command, name: command_dir.to_string_lossy().to_string() })?
            .to_string();

        validate_name(name.clone(), Entity::Command)?;

        let format = command_dir
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self { name, dir: command_dir, format })
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
