use std::path::PathBuf;
use anyhow::Result;
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
            strategies: Self::resolve_strategies(config.strategies_dir.clone())?,
            modules: vec![],
            config,
            executor: Executor::new()
        };
        client.modules = client.resolve_modules()?;
        Ok(client)
    }

    pub fn resolve_strategies(strategies_dir: PathBuf) -> Result<Vec<Strategy>> {
        strategies_dir.exists() && strategies_dir.is_dir() || return Err(anyhow::anyhow!("Invalid strategies directory"));
        Ok(sorted_read_dir(&strategies_dir)?
            .into_iter()
            .filter(|e| e.path().is_dir() && !is_ignored_name(&e.file_name().to_string_lossy()))
            .filter_map(|e| Strategy::new(e.path()).ok())
            .collect())
    }

    pub fn resolve_modules(&self) -> Result<Vec<Module>> {
        let mut modules: Vec<Module> = Vec::new();
        let mut walker = WalkDir::new(&self.config.modules_dir).into_iter();

        while let Some(entry) = walker.next() {
            let entry = entry?;

            if !entry.file_type().is_dir() {
                continue;
            }

            println!("{} - {:?}", entry.path().to_string_lossy(), entry.file_type());
            println!("len {}", self.strategies.len());
            let path = entry.path();
            let mut matched = false;

            for (i, strategy) in self.strategies.iter().enumerate() {
                println!("strategy: {} (tc {})", strategy.name, strategy.templates.len());
                for template in &strategy.templates {
                    println!("template: {} {}", strategy.name, template.name);
                    if template.matches(&path.to_path_buf()) {
                        matched = true;
                        if let Some(module) = modules.iter_mut().find(|m| m.dir == path) {
                            module.strategies.push(i);
                        } else {
                            modules.push(Module::new(path.to_path_buf(), vec![i]));
                        }
                    }
                }
            }

            if matched {
                walker.skip_current_dir();
            }
        }

        Ok(modules)
    }

    pub fn strategies_for_module(&self, path: &PathBuf) -> Vec<Strategy> {
        if let Some(module) = self.modules.iter().find(|m| m.dir == *path) {
            module.strategies.iter()
                .map(|&i| self.strategies[i].clone())
                .collect::<Vec<Strategy>>()
        } else {
            vec![]
        }
    }

    pub fn get_strategy(&self, name: &String) -> Option<Strategy> {
        self.strategies.iter().find(|s| s.name.eq(name)).cloned()
    }

    pub fn get_templates(&self) -> Vec<Template> {
        self.strategies.iter().flat_map(|s| s.templates.clone()).collect()
    }

    pub fn get_default_strategies(&self) -> Vec<Strategy> {
        self.strategies.iter().filter(|s| s.templates.is_empty()).cloned().collect()
    }
}