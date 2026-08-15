use std::path::PathBuf;

use anyhow::Result;

use crate::v2::config::MoldXConfig;
use crate::v2::strategy::{Strategy};
use crate::v2::fs::{sorted_read_dir, is_ignored_name};
use crate::v2::template::Template;

#[derive(Debug, Clone)]
pub struct MoldXClient {
    pub strategies: Vec<Strategy>,
    pub config: MoldXConfig,
}

impl MoldXClient {
    pub fn new(config: MoldXConfig) -> Result<Self> {
        Ok(MoldXClient {
            strategies: Self::resolve_strategies(config.strategies_dir.clone())?,
            config
        })
    }

    pub fn resolve_strategies(strategies_dir: PathBuf) -> Result<Vec<Strategy>> {
        strategies_dir.exists() && strategies_dir.is_dir() || return Err(anyhow::anyhow!("Invalid strategies directory"));
        Ok(sorted_read_dir(&strategies_dir)?
            .into_iter()
            .filter(|e| e.path().is_dir() && !is_ignored_name(&e.file_name().to_string_lossy()))
            .filter_map(|e| Strategy::new(e.path()).ok())
            .collect())
    }

    pub fn strategies_for_module(&self, path: &PathBuf) -> Vec<Strategy> {
        self.strategies.iter().filter(|s| s.clone().clone().available_for(path)).cloned().collect()
    }

    pub fn get_strategy(&self, name: &String) -> Option<Strategy> {
        self.strategies.iter().find(|s| s.name.eq(name)).cloned()
    }

    pub fn discover_modules(&self) -> Result<Vec<String>> {
        &self.config.cwd.clone();
        let templates = *self.get_templates();
        Ok(vec![])
    }

    pub fn get_templates(self) -> Vec<Template> {
        self.strategies.iter().flat_map(|s| s.templates.clone()).collect()
    }
}