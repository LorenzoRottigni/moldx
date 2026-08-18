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
