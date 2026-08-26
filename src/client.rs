use crate::config::MoldXConfig;
use crate::executor::Executor;
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
            profiles: Profile::resolve_profiles(&config.profiles_dir, &config)?,
            modules: vec![],
            config,
            executor: Executor::new(),
        };
        client.load_modules()?;
        Ok(client)
    }

    fn load_modules(&mut self) -> Result<()> {
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

    pub fn exec() -> Result<()> {
        // find a way to associate handler mod resolution with enum values
        Ok(())
    }
}
