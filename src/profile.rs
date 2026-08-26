use crate::{
    command::Command, config::MoldXConfig, errors::{MoldXError2}, fs::{resolve_name, sorted_read_dir}, template::Template, types::Entity,
};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// A named collection of templates, commands, and nested profiles.
///
/// A `Profile` represents a single configuration variant (e.g. `docker` or
/// `podman`). It owns a template that determines which modules it applies
/// to, a set of executable commands, and optional nested child profiles.
#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub template: Template,
    pub commands: Vec<Command>,
    pub profiles: Vec<Profile>,
}

impl Profile {
    /// Loads a profile from a directory.
    ///
    /// Reads the profile's template, commands, and nested child profiles,
    /// then validates that every child's template is a superset of this
    /// profile's template.
    ///
    /// # Arguments
    ///
    /// * `path` - The profile directory.
    /// * `config` - The MoldX configuration providing directory names.
    ///
    /// # Returns
    ///
    /// The fully resolved profile.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a directory, the name cannot be
    /// resolved, or child profile validation fails.
    pub fn new(path: &Path, config: &MoldXConfig) -> Result<Self> {
        if !path.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: path.to_path_buf(),
                kind: "profile",
            });
        }

        let name = resolve_name(path, Entity::Profile)?;
        let template = Template::new(path.join(&config.template_dir_name))?;
        let commands = Command::resolve_commands(&path.join(&config.bin_dir_name))?;
        let profiles = Self::resolve_profiles(&path.join(&config.profiles_dir_name), config)?;

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

    /// Discovers and loads all profiles contained in a directory.
    ///
    /// Each immediate child directory is treated as a profile. When the
    /// source directory does not exist, an empty list is returned.
    ///
    /// # Arguments
    ///
    /// * `source` - The directory to scan for profile subdirectories.
    /// * `config` - The MoldX configuration.
    ///
    /// # Returns
    ///
    /// The list of resolved profiles, sorted by directory name.
    ///
    /// # Errors
    ///
    /// Returns an error if a profile directory cannot be read or loaded.
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

    /// Looks up a command by name in this profile only.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name to search for.
    ///
    /// # Returns
    ///
    /// The matching command, or `None` if not found locally.
    pub fn get_local_command(&self, name: &String) -> Option<Command> {
        self.commands.iter().find(|c| c.name == *name).cloned()
    }

    /// Looks up a command by name, searching child profiles first and then
    /// this profile's own commands.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name to search for.
    ///
    /// # Returns
    ///
    /// The first matching command found in the profile tree, or `None`.
    pub fn get_command(&self, name: &String) -> Option<Command> {
        self.profiles
            .iter()
            .find_map(|p| p.get_command(name))
            .or(self.get_local_command(name))
    }

    /// Returns whether this profile's template is a superset of the
    /// given profile's template, meaning this profile can act as a child.
    ///
    /// # Arguments
    ///
    /// * `profile` - The potential parent profile.
    ///
    /// # Returns
    ///
    /// `true` when this profile is a valid child.
    pub fn is_child_of(&self, profile: &Profile) -> bool {
        self.template.is_child_of(&profile.template)
    }

    /// Verifies that every nested child profile's template is a superset
    /// of this profile's template.
    ///
    /// # Returns
    ///
    /// Ok when all children are valid.
    ///
    /// # Errors
    ///
    /// Returns [`MoldXError2::UnmatchedChildProfile`] if a child's template
    /// does not include all of this profile's template files.
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
