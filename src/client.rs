use crate::config::MoldXConfig;
use crate::executor::Executor;
use crate::module::Module;
use crate::profile::Profile;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use walkdir::WalkDir;

/// Main facade for interacting with a MoldX project.
///
/// A `MoldXClient` loads profiles, discovers modules, and provides
/// commands for executing profile scripts against modules.
#[derive(Debug)]
pub struct MoldXClient {
    pub profiles: Vec<Profile>,
    pub modules: Vec<Module>,
    pub config: MoldXConfig,
    pub executor: Executor,
}

impl MoldXClient {
    /// Builds a new client by loading profiles and discovering modules.
    ///
    /// # Arguments
    ///
    /// * `config` - The resolved MoldX configuration.
    ///
    /// # Returns
    ///
    /// A fully initialized client ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if profiles or modules cannot be loaded.
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

    /// Walks the modules directory and populates `self.modules`.
    ///
    /// Only directories that match at least one profile's template are
    /// kept. Directories inside the `.moldx` tree are skipped.
    ///
    /// # Returns
    ///
    /// Ok once the module list has been refreshed.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory tree cannot be traversed.
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

            if let Ok(module) = Module::resolve(path, &self.profiles)
                && !module.profiles.is_empty()
            {
                self.modules.push(module);
            }
        }

        self.modules.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(())
    }

    /// Walks the modules directory and returns matched modules without
    /// mutating the client.
    ///
    /// Behaves identically to [`load_modules`] but returns the result
    /// instead of storing it.
    ///
    /// # Returns
    ///
    /// The list of discovered modules, sorted by path.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory tree cannot be traversed.
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

            if let Ok(module) = Module::resolve(path, &self.profiles)
                && !module.profiles.is_empty()
            {
                modules.push(module);
            }
        }

        modules.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(modules)
    }

    /// Returns all profiles whose templates match the given module path.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path to test against profile templates.
    ///
    /// # Returns
    ///
    /// References to the matching profiles.
    pub fn profiles_for_module(&self, module_path: &std::path::Path) -> Vec<&Profile> {
        self.profiles
            .iter()
            .filter(|p| p.template.matches(module_path))
            .collect()
    }

    /// Placeholder for future command-handler dispatch logic.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn exec() -> Result<()> {
        // find a way to associate handler mod resolution with enum values
        Ok(())
    }
}

/// Prints the configuration, executor status, profiles, and modules.
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
