use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::path::Path;
use anyhow::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::MoldXConfig;
use crate::errors::MoldXError;
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
        if !strategies_dir.exists() || !strategies_dir.is_dir() {
            return Err(MoldXError::InvalidStrategiesDir { path: strategies_dir.clone() }.into());
        }
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

            if let Ok(module) = Module::new(canonical_path, matching_strategies.into_iter().collect()) {
                modules.push(module);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::path::PathBuf;

    fn setup_client(dir: &std::path::Path) -> MoldXClient {
        let moldx_dir = dir.join(".moldx");
        let strategies_dir = moldx_dir.join("strategies");
        fs::create_dir_all(&strategies_dir).unwrap();

        let config = MoldXConfig {
            moldx_dir: moldx_dir.clone(),
            strategies_dir: strategies_dir.clone(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: dir.to_path_buf(),
            max_resolution_depth: 20,
        };
        MoldXClient::new(config).unwrap()
    }

    #[test]
    fn test_client_new_empty() {
        let dir = tempdir().unwrap();
        let client = setup_client(dir.path());
        assert!(client.strategies.is_empty());
        assert!(client.modules.is_empty());
    }

    #[test]
    fn test_client_new_with_strategy() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("bin")).unwrap();
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        fs::write(strategy_dir.join("bin").join("build.sh"), "#!/bin/bash").unwrap();
        let client = setup_client(dir.path());
        assert_eq!(client.strategies.len(), 1);
        assert_eq!(client.strategies[0].name, "docker");
    }

    #[test]
    fn test_resolve_strategies_invalid_dir() {
        let config = MoldXConfig {
            moldx_dir: PathBuf::from("/nonexistent"),
            strategies_dir: PathBuf::from("/nonexistent/strategies"),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: PathBuf::from("/nonexistent"),
            max_resolution_depth: 20,
        };
        let result = MoldXClient::resolve_strategies(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_modules_with_matching_module() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        fs::create_dir_all(dir.path().join("my-service")).unwrap();
        fs::write(dir.path().join("my-service").join("Dockerfile"), "").unwrap();
        let client = setup_client(dir.path());
        assert_eq!(client.modules.len(), 1);
        assert_eq!(client.modules[0].name, "my-service");
    }

    #[test]
    fn test_resolve_modules_skips_non_matching() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        fs::create_dir_all(dir.path().join("no-docker")).unwrap();
        fs::write(dir.path().join("no-docker").join("main.rs"), "").unwrap();
        let client = setup_client(dir.path());
        assert!(client.modules.is_empty());
    }

    #[test]
    fn test_resolve_modules_skips_ignored_dirs() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("node");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("package.json"), "").unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target").join("main.rs"), "").unwrap();
        let client = setup_client(dir.path());
        assert!(client.modules.is_empty());
    }

    #[test]
    fn test_resolve_modules_skips_moldx_dir() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("node");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("package.json"), "").unwrap();
        fs::create_dir_all(dir.path().join(".moldx").join("some-module")).unwrap();
        fs::write(dir.path().join(".moldx").join("some-module").join("package.json"), "").unwrap();
        let client = setup_client(dir.path());
        assert!(client.modules.is_empty());
    }

    #[test]
    fn test_strategies_for_module() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        fs::create_dir_all(dir.path().join("my-service")).unwrap();
        fs::write(dir.path().join("my-service").join("Dockerfile"), "").unwrap();
        let client = setup_client(dir.path());
        let strategies = client.strategies_for_module(&dir.path().join("my-service"));
        assert_eq!(strategies.len(), 1);
        assert_eq!(strategies[0].name, "docker");
    }

    #[test]
    fn test_strategies_for_module_includes_agnostic() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let default_dir = strategies_dir.join("default");
        fs::create_dir_all(default_dir.join("bin")).unwrap();
        fs::write(default_dir.join("bin").join("diff.sh"), "#!/bin/bash").unwrap();
        fs::create_dir_all(default_dir.join("template")).unwrap();
        fs::write(default_dir.join("template").join(".gitkeep"), "").unwrap();
        let client = setup_client(dir.path());
        let strategies = client.strategies_for_module(&dir.path().join("anywhere"));
        assert_eq!(strategies.len(), 1);
        assert_eq!(strategies[0].name, "default");
    }

    #[test]
    fn test_get_strategy_found() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        fs::create_dir_all(strategies_dir.join("default")).unwrap();
        fs::create_dir_all(strategies_dir.join("default").join("template")).unwrap();
        fs::write(strategies_dir.join("default").join("template").join(".gitkeep"), "").unwrap();
        let client = setup_client(dir.path());
        assert!(client.get_strategy(&"default".into()).is_some());
    }

    #[test]
    fn test_get_strategy_not_found() {
        let dir = tempdir().unwrap();
        let client = setup_client(dir.path());
        assert!(client.get_strategy(&"nope".into()).is_none());
    }

    #[test]
    fn test_get_templates() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        let client = setup_client(dir.path());
        let templates = client.get_templates();
        assert_eq!(templates.len(), 1);
    }

    #[test]
    fn test_get_default_strategies() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let agnostic = strategies_dir.join("default");
        fs::create_dir_all(agnostic.join("template")).unwrap();
        fs::write(agnostic.join("template").join(".gitkeep"), "").unwrap();
        let non_agnostic = strategies_dir.join("docker");
        fs::create_dir_all(non_agnostic.join("template")).unwrap();
        fs::write(non_agnostic.join("template").join("Dockerfile"), "").unwrap();
        let client = setup_client(dir.path());
        let defaults = client.get_default_strategies();
        assert_eq!(defaults.len(), 1);
        assert_eq!(defaults[0].name, "default");
    }

    #[test]
    fn test_client_display() {
        let dir = tempdir().unwrap();
        let client = setup_client(dir.path());
        let display = client.to_string();
        assert!(display.contains("MoldX Snapshot"));
        assert!(display.contains("config"));
        assert!(display.contains("strategies"));
        assert!(display.contains("modules"));
        assert!(display.contains("executor"));
    }

    #[test]
    fn test_client_display_with_strategies_and_modules() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        fs::create_dir(dir.path().join("my-service")).unwrap();
        fs::write(dir.path().join("my-service").join("Dockerfile"), "").unwrap();
        let client = setup_client(dir.path());
        let display = client.to_string();
        assert!(display.contains("docker"));
        assert!(display.contains("my-service"));
    }
}
