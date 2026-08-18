use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::path::Path;
use anyhow::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::MoldXConfig;
use crate::executor::Executor;
use crate::strategy::{Strategy};
use crate::fs::{sorted_read_dir, is_ignored_name};
use crate::template::Template;
use crate::module::Module;

#[derive(Debug)]
pub struct MoldXClient {
    pub strategies: Vec<Strategy>,
    pub modules: Vec<Module>,
    pub config: MoldXConfig,
    pub executor: Executor
}

impl MoldXClient {
    pub fn new(config: MoldXConfig) -> Result<Self> {
        let mut client = MoldXClient {
            strategies: Self::resolve_strategies(&config)?,
            modules: vec![],
            config,
            executor: Executor::new()
        };
        client.modules = client.resolve_modules()?;
        Ok(client)
    }

    pub fn resolve_strategies(config: &MoldXConfig) -> Result<Vec<Strategy>> {
        let strategies_dir = &config.strategies_dir;
        strategies_dir.exists() && strategies_dir.is_dir() || return Err(anyhow::anyhow!("Invalid strategies directory"));
        Ok(sorted_read_dir(strategies_dir)?
            .into_iter()
            .filter(|e| e.path().is_dir() && !is_ignored_name(&e.file_name().to_string_lossy()))
            .map(|e| Strategy::new(e.path(), config))
            .collect::<Result<Vec<_>>>()?)
    }

    pub fn resolve_modules(&self) -> Result<Vec<Module>> {
        let mut modules: Vec<Module> = Vec::new();
        let moldx_dir = self
            .config
            .moldx_dir
            .canonicalize()
            .unwrap_or_else(|_| self.config.moldx_dir.clone());
        let mut walker = WalkDir::new(&self.config.modules_dir).into_iter();

        while let Some(entry) = walker.next() {
            let entry = entry?;

            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

            if canonical_path.starts_with(&moldx_dir) {
                walker.skip_current_dir();
                continue;
            }

            let matching_strategies = self
                .strategies
                .iter()
                .enumerate()
                .filter(|(_, strategy)| !strategy.is_agnostic())
                .filter(|(_, strategy)| strategy.templates.iter().any(|template| template.matches(path)))
                .map(|(index, _)| index)
                .collect::<BTreeSet<_>>();

            if matching_strategies.is_empty() {
                continue;
            }

            modules.push(Module::new(canonical_path, matching_strategies.into_iter().collect()));
        }

        modules.sort_by(|a, b| a.dir.cmp(&b.dir));
        Ok(modules)
    }

    pub fn strategies_for_module(&self, path: &Path) -> Vec<Strategy> {
        let resolved_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let mut strategies = if let Some(module) = self.modules.iter().find(|m| m.dir == resolved_path) {
            module
                .strategies
                .iter()
                .map(|&i| self.strategies[i].clone())
                .collect::<Vec<Strategy>>()
        } else {
            Vec::new()
        };

        for strategy in self.get_default_strategies() {
            if !strategies.iter().any(|existing| existing.name == strategy.name) {
                strategies.push(strategy);
            }
        }

        strategies
    }

    pub fn get_strategy(&self, name: &String) -> Option<Strategy> {
        self.strategies.iter().find(|s| s.name.eq(name)).cloned()
    }

    pub fn get_templates(&self) -> Vec<Template> {
        self.strategies.iter().flat_map(|s| s.templates.clone()).collect()
    }

    pub fn get_default_strategies(&self) -> Vec<Strategy> {
        self.strategies.iter().filter(|s| s.is_agnostic()).cloned().collect()
    }
}

impl Display for MoldXClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", "╭─".dimmed(), "MoldX Snapshot".bold().cyan())?;
        writeln!(f, "{} {}", "│".dimmed(), "config".bold().yellow())?;
        for line in self.config.to_string().lines() {
            writeln!(f, "{}   {}", "│".dimmed(), line)?;
        }

        writeln!(f, "{}", "│".dimmed())?;
        writeln!(f, "{} {} ({})", "│".dimmed(), "strategies".bold().yellow(), self.strategies.len())?;
        if self.strategies.is_empty() {
            writeln!(f, "{}   {}", "│".dimmed(), "none".italic().dimmed())?;
        } else {
            for strategy in &self.strategies {
                for line in strategy.to_string().lines() {
                    writeln!(f, "{}   {}", "│".dimmed(), line)?;
                }
            }
        }

        writeln!(f, "{}", "│".dimmed())?;
        writeln!(f, "{} {} ({})", "│".dimmed(), "modules".bold().yellow(), self.modules.len())?;
        if self.modules.is_empty() {
            writeln!(f, "{}   {}", "│".dimmed(), "none".italic().dimmed())?;
        } else {
            for module in &self.modules {
                writeln!(f, "{}   {}", "│".dimmed(), module)?;
            }
        }

        writeln!(f, "{}", "│".dimmed())?;
        writeln!(f, "{} {}", "│".dimmed(), "executor".bold().yellow())?;
        for line in self.executor.to_string().lines() {
            writeln!(f, "{}   {}", "│".dimmed(), line)?;
        }

        writeln!(f, "{}", "╰".dimmed())?;
        Ok(())
    }
}
