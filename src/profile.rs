use crate::{
    command::Command, config::MoldXConfig, errors::{MoldXError2}, fs::{resolve_name, sorted_read_dir}, template::Template, types::Entity,
};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub template: Template,
    pub commands: Vec<Command>,
    pub profiles: Vec<Profile>,
}

impl Profile {
    pub fn new(path: &Path, config: &MoldXConfig) -> Result<Self> {
        if !path.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: path.to_path_buf(),
                kind: "profile",
            });
        }

        let name = resolve_name(path, Entity::Profile)?;
        let template = Self::resolve_template(&path.join(&config.template_dir_name))?;
        let commands = Self::resolve_commands(&path.join(&config.bin_dir_name))?;
        let profiles = Self::resolve_profiles(&path.join("profiles"), config)?;

        let profile = Self {
            name,
            path: path.to_path_buf(),
            template,
            commands,
            profiles,
        };

        profile.validate_children()?;

        Ok(profile)
    }

    pub fn resolve_commands(source: &Path) -> Result<Vec<Command>> {
        if !source.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: source.to_path_buf(),
                kind: "profile bin",
            });
        }

        sorted_read_dir(source)?
            .into_iter()
            .filter(|e| e.path().is_file())
            .map(|e| Command::new(e.path()))
            .collect()
    }

    pub fn resolve_template(source: &Path) -> Result<Template> {
        if !source.exists() || !source.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: source.to_path_buf(),
                kind: "profile template"
            })
        }
        Ok(Template::new(source.to_path_buf())?)
    }

    pub fn resolve_profiles(
        source: &Path,
        config: &MoldXConfig,
    ) -> Result<Vec<Self>> {
        if !source.is_dir() {
            return Ok(vec![]);
        }

        sorted_read_dir(source)?
            .into_iter()
            .filter(|e| e.path().is_dir())
            .map(|e| Profile::new(&e.path(), config))
            .collect()
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
        self.template.is_child_of(&profile.template)
    }

    pub fn validate_children(&self) -> Result<()> {
        if let Some(child) = self.profiles.iter().find(|p| !p.is_child_of(self)) {
            bail!(MoldXError2::UnmatchedChildProfile {
                parent: self.path.clone(),
                child: child.path.clone(),
            });
        }

        Ok(())
    }
}
