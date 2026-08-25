use std::collections::BTreeSet;

use crate::config::MoldXConfig;
use crate::errors::MoldXError;
use crate::executor::Executor;
use crate::fs::{is_ignored_name, sorted_read_dir};
use crate::module::Module;
use crate::profile::Profile;

use anyhow::Result;
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
            profiles: vec![],
            modules: vec![],
            config,
            executor: Executor::new(),
        };
        client.load_profiles();
        client.load_modules();
        Ok(client)
    }

    pub fn load_profiles(&mut self) -> Result<()> {
        let profiles_dir = &self.config.strategies_dir;
        if !profiles_dir.exists() || !profiles_dir.is_dir() {
            return Err(MoldXError::InvalidStrategiesDir {
                path: profiles_dir.clone(),
            }
            .into());
        }
        self.profiles = sorted_read_dir(profiles_dir)?
            .into_iter()
            .filter(|e| e.path().is_dir() && !is_ignored_name(&e.file_name().to_string_lossy()))
            .map(|e| Profile::new(e.path(), &self.config))
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    pub fn load_modules(&mut self) -> Result<()> {
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
                .profiles
                .iter()
                .enumerate()
                .filter(|(_, profile)| profile.template.as_ref().is_some_and(|t| t.matches(path)))
                .map(|(index, _)| index)
                .collect::<BTreeSet<_>>();

            if matching_strategies.is_empty() {
                continue;
            }

            if let Ok(module) =
                Module::new(canonical_path, matching_strategies.into_iter().collect())
            {
                modules.push(module);
            }
        }

        modules.sort_by(|a, b| a.dir.cmp(&b.dir));

        self.modules = modules;

        Ok(())
    }

    pub fn exec() -> Result<()> {
        // find a way to associate handler mod resolution with enum values
        Ok(())
    }
}
