use std::path::PathBuf;
use crate::{command::Command, config::MoldXConfig, errors::MoldXError, fs::{sorted_read_dir, validate_name}, template::Template, types::Entity};
use anyhow::Result;

struct Profile {
    pub name: String,
    pub dir: PathBuf,
    pub template: Option<Template>,
    pub commands: Vec<Command>,
    pub profiles: Vec<Profile>
}

impl Profile {
    pub fn new(path: PathBuf, config: MoldXConfig) -> Result<Self> {
        if !path.exists() || !path.is_dir() {
            return Err(MoldXError::InvalidStrategyDir { path }.into());
        }
        let name = path
            .file_name()
            .ok_or_else(|| MoldXError::StrategyDirNoFileName { path: path.clone() })?
            .to_string_lossy()
            .into_owned();

        validate_name(name.clone(), Entity::Strategy)?;

        let mut profile = Self {
            dir: path,
            name,
            template: None,
            profiles: vec![],
            commands: vec![],
        };

        profile.load_commands(&config)?;
        profile.load_profiles(&config)?;
        profile.load_template(&config)?;

        Ok(profile)
    }

    pub fn load_commands(&mut self, config: &MoldXConfig) -> Result<()> {
        let commands_dir = self.dir.join(&config.bin_dir_name);
        if !commands_dir.is_dir() {
            return Err(MoldXError::PathNotFound { path: commands_dir }.into())
        }

        self.commands = sorted_read_dir(&self.dir)?
            .into_iter()
            .filter_map(|e| e.path().is_file().then(
                || Command::new(e.path()).ok()).flatten()
            )
            .collect();

        Ok(())
    }

    pub fn load_template(&mut self, config: &MoldXConfig) -> Result<()> {
        let template_dir = self.dir.join(&config.template_dir_name);
        if template_dir.is_dir() {
            self.template = Some(Template::new(config.template_dir_name.to_string(), template_dir)?);
        }
        Ok(())
    }

    pub fn load_profiles(&mut self, config: &MoldXConfig) -> Result<()> {
        let profiles_dir = self.dir.join("profiles");
        if profiles_dir.is_dir() {
            self.profiles = sorted_read_dir(&profiles_dir)?
                .into_iter()
                .filter_map(|e| e.path().is_file().then(
                    || Profile::new(e.path(), config.clone()).ok()).flatten()
                )
                .collect();
        }
        Ok(())
    }

    pub fn get_local_command(&self, name: &String) -> Option<Command> {
        self.commands.iter().find(|c| c.name == *name).cloned()
    }

    pub fn get_command(&self, name: &String) -> Option<Command> {
        self.profiles.iter().find_map(|p| p.get_command(name)).or(self.get_local_command(name))
    }
}
