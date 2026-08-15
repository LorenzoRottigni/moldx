use std::fmt::{self, Display};
use std::path::PathBuf;
use anyhow::{Result};

use crate::v2::fs::{sorted_read_dir};
use crate::v2::template::{Template};
use crate::v2::command::{Command};

#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub dir: PathBuf,
    pub commands: Vec<Command>,
    pub templates: Vec<Template>,
}

impl Strategy {
    pub fn new(strategy_dir: PathBuf) -> Result<Self> {
        strategy_dir.exists() && strategy_dir.is_dir() ||
            return Err(anyhow::anyhow!("Invalid strategy directory"));
        let name = strategy_dir
            .file_name()
            .expect("Strategy directory has no file name")
            .to_string_lossy()
            .into_owned();
        let commands = Self::resolve_commands(&strategy_dir)?;
        let templates = Self::resolve_templates(&strategy_dir)?;
        Ok(Self {
            dir: strategy_dir,
            name,
            commands,
            templates,
        })
    }

    pub fn resolve_commands(strategy_dir: &PathBuf) -> Result<Vec<Command>> {
        let commands_dir = strategy_dir.join("bin");
        commands_dir.exists() && commands_dir.is_dir() ||
            return Err(anyhow::anyhow!("Invalid strategy commands directory"));
        Ok(sorted_read_dir(&commands_dir)?
            .into_iter()
            .filter(|e| e.path().is_file())
            .filter_map(|e| Command::new(e.path()))
            .collect())
    }

    pub fn resolve_templates(strategy_dir: &PathBuf) -> Result<Vec<Template>> {
        [
            strategy_dir.join("template"),
            strategy_dir.join("templates"),
        ]
            .into_iter()
            .filter(|dir| dir.is_dir())
            .map(|dir| {
                sorted_read_dir(&dir).map(|entries| {
                    entries
                        .into_iter()
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| {
                            Template::new(
                                e.file_name().to_string_lossy().into_owned(),
                                e.path(),
                            ).ok()
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>>>()
            .map(|templates| templates.into_iter().flatten().collect())
    }

    pub fn get_command(&self, name: &String) -> Option<Command> {
        self.commands.iter().find(|c| c.name == *name).cloned()
    }
}

impl Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}