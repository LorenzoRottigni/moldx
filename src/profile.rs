use crate::{
    command::Command,
    config::MoldXConfig,
    errors::MoldXError,
    fs::{sorted_read_dir, validate_dir, validate_name},
    template::Template,
    types::Entity,
};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub dir: PathBuf,
    pub template: Option<Template>,
    pub commands: Vec<Command>,
    pub profiles: Vec<Profile>,
}

impl Profile {
    pub fn new(dir: PathBuf, config: &MoldXConfig) -> Result<Self> {
        let name = dir
            .file_name()
            .ok_or_else(|| MoldXError::StrategyDirNoFileName { path: dir.clone() })?
            .to_string_lossy()
            .into_owned();

        validate_name(name.clone(), Entity::Profile)?;

        let mut profile = Self {
            dir,
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

    pub fn resolve_commands(bin_dir: &PathBuf) -> Result<Vec<Command>> {
        Ok(sorted_read_dir(bin_dir)?
            .into_iter()
            .filter_map(|e| {
                e.path()
                    .is_file()
                    .then(|| Command::new(e.path()).ok())
                    .flatten()
            })
            .collect())
    }

    pub fn load_commands(&mut self, config: &MoldXConfig) -> Result<()> {
        let commands_dir = self.dir.join(&config.bin_dir_name);
        if !commands_dir.is_dir() {
            return Err(MoldXError::PathNotFound { path: commands_dir }.into());
        }

        self.commands = sorted_read_dir(&self.dir)?
            .into_iter()
            .filter_map(|e| {
                e.path()
                    .is_file()
                    .then(|| Command::new(e.path()).ok())
                    .flatten()
            })
            .collect();

        Ok(())
    }

    pub fn load_template(&mut self, config: &MoldXConfig) -> Result<()> {
        let template_dir = self.dir.join(&config.template_dir_name);

        // no template dir throw err
        // templates hireachy unmatch throw err
        if !template_dir.is_dir() {
            return Err(MoldXError::TemplateNotFound {
                strategy: self.name.clone(),
            }
            .into());
        }

        if template_dir.is_dir() {
            self.template = Some(Template::new(template_dir)?);
        }
        Ok(())
    }

    pub fn load_profiles(&mut self, config: &MoldXConfig) -> Result<()> {
        let profiles_dir = self.dir.join("profiles");
        if profiles_dir.is_dir() {
            self.profiles = sorted_read_dir(&profiles_dir)?
                .into_iter()
                .filter_map(|e| {
                    e.path()
                        .is_file()
                        .then(|| Profile::new(e.path(), config).ok())
                        .flatten()
                })
                .collect();
        }
        Ok(())
    }

    pub fn get_local_command(&self, name: &String) -> Option<Command> {
        self.commands.iter().find(|c| c.name == *name).cloned()
    }

    pub fn get_command(&self, name: &String) -> Option<Command> {
        self.profiles
            .iter()
            .find_map(|p| p.get_command(name))
            .or(self.get_local_command(name))
    }

    pub fn is_child_of(&self, profile: &Profile) -> bool {
        match (&self.template, &profile.template) {
            (Some(template), Some(parent_template)) => template.is_child_of(parent_template),
            _ => false,
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_dir(&self.dir)?;

        self.template
            .as_ref()
            .ok_or_else(|| MoldXError::TemplateNotFound {
                strategy: self.name.clone(),
            })?
            .validate()?;

        for profile in &self.profiles {
            profile.validate()?;
        }

        for command in &self.commands {
            command.validate()?;
        }

        self.validate_children()?;

        Ok(())
    }

    pub fn validate_children(&self) -> Result<()> {
        if !self.profiles.iter().all(|p| p.is_child_of(self)) {
            return Err(MoldXError::PhantomChildren {
                path: self.dir.clone(),
            }
            .into());
        }

        Ok(())
    }
}
