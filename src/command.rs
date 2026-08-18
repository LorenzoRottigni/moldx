use std::path::PathBuf;
use std::fmt::{self, Display};
use owo_colors::OwoColorize;

use crate::fs::is_shell_script;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub dir: PathBuf,
    pub format: String,
}

impl Command {
    pub fn new(command_dir: PathBuf) -> Option<Self> {
        if !command_dir.is_file() || !is_shell_script(&command_dir) {
            return None;
        }

        let name = command_dir
            .file_stem()
            .and_then(|stem| stem.to_str())?
            .to_string();

        let format = command_dir
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        Some(Self { name, dir: command_dir, format })
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
