use crate::{
    command::Command,
    config::MoldXConfig,
    errors::MoldXError2,
    fs::{resolve_name, sorted_read_dir},
    template::Template,
    types::Entity,
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
    /// Builds the root profile represented by `.moldx`.
    ///
    /// The root profile owns commands from `.moldx/bin` and contains the
    /// technology profiles loaded from `.moldx/profiles`.
    pub fn root(path: &Path, config: &MoldXConfig) -> Result<Self> {
        let bin_dir = path.join(&config.bin_dir_name);
        let commands = if bin_dir.is_dir() {
            Command::resolve_commands(&bin_dir)?
        } else {
            Vec::new()
        };
        let profiles = Self::resolve_profiles(&config.profiles_dir, config)?;

        let profile = Self {
            name: "root".to_string(),
            path: path.to_path_buf(),
            template: Template {
                path: path.to_path_buf(),
                file_names: Default::default(),
            },
            commands,
            profiles,
        };
        profile.validate_children()?;
        Ok(profile)
    }

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
    pub fn resolve_profiles(source: &Path, config: &MoldXConfig) -> Result<Vec<Self>> {
        if !source.is_dir() {
            return Ok(vec![]);
        }

        sorted_read_dir(source)?
            .into_iter()
            .filter(|e| e.path().is_dir())
            .map(|e| Profile::new(&e.path(), config))
            .collect()
    }

    /// Recursively discovers commands matching a module and optional profile
    /// hierarchy.
    ///
    /// Profile names are resolved from top to bottom. When `profile_names` is
    /// empty, all profiles are traversed at the current level. Otherwise, only
    /// the profile matching the first name is traversed, with the remaining names
    /// passed recursively to child profiles.
    ///
    /// A command is added to `discovered` when the current profile matches the
    /// requested hierarchy and exposes the requested command.
    pub fn commands_for_module(
        &self,
        command_name: &str,
        module_path: &Path,
        discovered: &mut Vec<Command>,
        profile_names: &[String],
    ) {
        if self.template.matches(module_path)
            && profile_names.first().is_none_or(|n| self.name == *n)
            && let Some(command) = self.get_local_command(&command_name.to_string())
        {
            discovered.push(command);
        }

        self.profiles
            .iter()
            .filter(|p| p.template.matches(module_path))
            .filter(|p| profile_names.first().is_none_or(|n| p.name == *n))
            .for_each(|p| {
                p.commands_for_module(
                    command_name,
                    module_path,
                    discovered,
                    profile_names.get(1..).unwrap_or_default(),
                );
            });
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_config() -> MoldXConfig {
        MoldXConfig {
            moldx_dir: PathBuf::from("/nonexistent/.moldx"),
            profiles_dir: PathBuf::from("/nonexistent/.moldx/profiles"),
            profiles_dir_name: "profiles".into(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: PathBuf::from("/nonexistent"),
            max_resolution_depth: 20,
        }
    }

    fn write(profile_dir: &Path, files: &[&str], commands: &[&str]) {
        let bin = profile_dir.join("bin");
        fs::create_dir_all(&bin).unwrap();
        for c in commands {
            fs::write(bin.join(format!("{}.sh", c)), "#!/usr/bin/env bash\nexit 0").unwrap();
        }
        let template = profile_dir.join("template");
        fs::create_dir_all(&template).unwrap();
        for f in files {
            fs::write(template.join(f), "").unwrap();
        }
    }

    #[test]
    fn test_get_command_local() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("docker");
        write(&pdir, &["Dockerfile"], &["build", "test"]);
        let profile = Profile::new(&pdir, &make_config()).unwrap();
        assert_eq!(
            profile.get_command(&"build".to_string()).unwrap().name,
            "build"
        );
    }

    #[test]
    fn test_get_command_missing() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("docker");
        write(&pdir, &["Dockerfile"], &["build"]);
        let profile = Profile::new(&pdir, &make_config()).unwrap();
        assert!(profile.get_command(&"nope".to_string()).is_none());
    }

    #[test]
    fn test_get_command_searches_children() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("node");
        write(&pdir, &["package.json"], &[]);
        let nuxt = pdir.join("profiles").join("nuxt");
        write(&nuxt, &["package.json", "nuxt.config.ts"], &["dev"]);
        let profile = Profile::new(&pdir, &make_config()).unwrap();
        assert_eq!(profile.get_command(&"dev".to_string()).unwrap().name, "dev");
    }

    #[test]
    fn test_commands_for_module_resolves_local() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("docker");
        write(&pdir, &["Dockerfile"], &["build"]);
        let profile = Profile::new(&pdir, &make_config()).unwrap();

        let module = dir.path().join("svc");
        fs::create_dir(&module).unwrap();
        fs::write(module.join("Dockerfile"), "").unwrap();

        let mut found = Vec::new();
        profile.commands_for_module("build", &module, &mut found, &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "build");
    }

    #[test]
    fn test_commands_for_module_ignores_non_matching_module() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("docker");
        write(&pdir, &["Dockerfile"], &["build"]);
        let profile = Profile::new(&pdir, &make_config()).unwrap();

        let module = dir.path().join("svc");
        fs::create_dir(&module).unwrap();

        let mut found = Vec::new();
        profile.commands_for_module("build", &module, &mut found, &[]);
        assert!(found.is_empty());
    }

    #[test]
    fn test_commands_for_module_filters_by_profile_name() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("docker");
        write(&pdir, &["Dockerfile"], &["build"]);
        let profile = Profile::new(&pdir, &make_config()).unwrap();

        let module = dir.path().join("svc");
        fs::create_dir(&module).unwrap();
        fs::write(module.join("Dockerfile"), "").unwrap();

        let mut found = Vec::new();
        profile.commands_for_module("build", &module, &mut found, &["other".into()]);
        assert!(found.is_empty());

        let mut found = Vec::new();
        profile.commands_for_module("build", &module, &mut found, &["docker".into()]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_commands_for_module_recurses_into_children() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("node");
        write(&pdir, &["package.json"], &[]);
        let nuxt = pdir.join("profiles").join("nuxt");
        write(
            &nuxt,
            &["package.json", "nuxt.config.ts"],
            &["dev", "start"],
        );
        let profile = Profile::new(&pdir, &make_config()).unwrap();

        let module = dir.path().join("app");
        fs::create_dir(&module).unwrap();
        fs::write(module.join("package.json"), "").unwrap();
        fs::write(module.join("nuxt.config.ts"), "").unwrap();

        let mut found = Vec::new();
        profile.commands_for_module("dev", &module, &mut found, &[]);
        assert_eq!(found.len(), 1);

        let mut found = Vec::new();
        profile.commands_for_module("dev", &module, &mut found, &["nuxt".into()]);
        assert_eq!(found.len(), 1);

        let mut found = Vec::new();
        profile.commands_for_module("start", &module, &mut found, &["other".into()]);
        assert!(found.is_empty());
    }

    #[test]
    fn test_validate_children_rejects_non_superset() {
        let dir = tempfile::tempdir().unwrap();
        let pdir = dir.path().join("node");
        // parent matches package.json
        write(&pdir, &["package.json"], &[]);
        let nuxt = pdir.join("profiles").join("nuxt");
        // child missing package.json -> not a superset -> invalid
        write(&nuxt, &["nuxt.config.ts"], &["dev"]);
        let result = Profile::new(&pdir, &make_config());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_child_of() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("node");
        write(&parent, &["package.json"], &[]);
        let child = dir.path().join("nuxt");
        write(&child, &["package.json", "nuxt.config.ts"], &[]);
        let p = Profile::new(&parent, &make_config()).unwrap();
        let c = Profile::new(&child, &make_config()).unwrap();
        assert!(c.is_child_of(&p));
        assert!(!p.is_child_of(&c));
    }
}
