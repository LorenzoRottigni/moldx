use crate::config::MoldXConfig;
use crate::executor::Executor;
use crate::module::Module;
use crate::profile::Profile;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct MoldXClient {
    pub profiles: Vec<Profile>,
    pub modules: Vec<Module>,
    pub config: MoldXConfig,
    pub executor: Executor,
}

impl MoldXClient {
    pub fn new(config: MoldXConfig) -> Result<Self> {
        let mut client = Self {
            profiles: Profile::resolve_profiles(&config.profiles_dir, &config)?,
            modules: vec![],
            config,
            executor: Executor::new(),
        };
        client.load_modules()?;
        Ok(client)
    }

    pub fn load_modules(&mut self) -> Result<()> {
        let moldx_dir = self
            .config
            .moldx_dir
            .canonicalize()
            .unwrap_or_else(|_| self.config.moldx_dir.clone());

        let mut walker = WalkDir::new(&self.config.modules_dir).into_iter();

        self.modules = Vec::new();

        while let Some(entry) = walker.next() {
            let entry = entry?;

            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry
                .path()
                .canonicalize()
                .unwrap_or_else(|_| entry.path().to_path_buf());

            if path.starts_with(&moldx_dir) {
                walker.skip_current_dir();
                continue;
            }

            if let Ok(module) = Module::resolve(path, &self.profiles) {
                if !module.profiles.is_empty() {
                    self.modules.push(module);
                }
            }
        }

        self.modules.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(())
    }

    pub fn resolve_modules(&self) -> Result<Vec<Module>> {
        let moldx_dir = self
            .config
            .moldx_dir
            .canonicalize()
            .unwrap_or_else(|_| self.config.moldx_dir.clone());

        let mut walker = WalkDir::new(&self.config.modules_dir).into_iter();
        let mut modules = Vec::new();

        while let Some(entry) = walker.next() {
            let entry = entry?;

            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry
                .path()
                .canonicalize()
                .unwrap_or_else(|_| entry.path().to_path_buf());

            if path.starts_with(&moldx_dir) {
                walker.skip_current_dir();
                continue;
            }

            if let Ok(module) = Module::resolve(path, &self.profiles) {
                if !module.profiles.is_empty() {
                    modules.push(module);
                }
            }
        }

        modules.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(modules)
    }

    pub fn profiles_for_module(&self, module_path: &std::path::Path) -> Vec<&Profile> {
        self.profiles
            .iter()
            .filter(|p| p.template.matches(module_path))
            .collect()
    }

    pub fn exec() -> Result<()> {
        // find a way to associate handler mod resolution with enum values
        Ok(())
    }
}

impl Display for MoldXClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.config)?;
        writeln!(f, "{}", self.executor)?;
        writeln!(f, "{} {}", "profiles:".bold().yellow(), self.profiles.len())?;
        for profile in &self.profiles {
            writeln!(f, "  {}", profile.name.bold().green())?;
        }
        writeln!(f, "{} {}", "modules:".bold().yellow(), self.modules.len())?;
        for module in &self.modules {
            writeln!(f, "  {}", module)?;
        }
        Ok(())
    }
}
