//! Facade for interacting with a resolved MoldX project.
//!
//! [`crate::client::MoldXClient`] ties together profiles, modules, and the executor,
//! providing discovery of modules and resolution of commands.

use crate::executor::Executor;
use crate::module::Module;
use crate::profile::Profile;
use crate::{command::Command, config::MoldXConfig};

use anyhow::Result;
use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::Path;
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
        let root_profile = Profile::root(&config.moldx_dir, &config)?;
        let mut client = Self {
            profiles: vec![root_profile],
            modules: vec![],
            config,
            executor: Executor::new(),
        };
        client.load_modules()?;
        Ok(client)
    }

    /// Builds a client for filesystem scaffolding without scanning modules.
    pub fn new_for_scaffolding(config: MoldXConfig) -> Result<Self> {
        let root_profile = Profile::root(&config.moldx_dir, &config)?;
        Ok(Self {
            profiles: vec![root_profile],
            modules: Vec::new(),
            config,
            executor: Executor::new(),
        })
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

            if let Ok(module) = Module::resolve(path, self.profile_children())
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
    /// Behaves identically to [`Self::load_modules`] but returns the result
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

            if let Ok(module) = Module::resolve(path, self.profile_children())
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
        self.profile_children()
            .iter()
            .filter(|p| p.template.matches(module_path))
            .collect()
    }

    /// Resolves every command named `command_name` that applies to a module,
    /// optionally restricted to a profile hierarchy.
    ///
    /// # Arguments
    ///
    /// * `command_name` - The name of the command to search for.
    /// * `module_path` - The module the command will be run against.
    /// * `profile_names` - Optional profile hierarchy to restrict the search;
    ///   an empty slice searches all profiles.
    ///
    /// # Returns
    ///
    /// The list of matching commands, in resolution order.
    pub fn commands_for_module(
        &self,
        command_name: &str,
        module_path: &Path,
        profile_names: &[String],
    ) -> Vec<Command> {
        let mut discovered = Vec::new();
        self.profiles[0].commands_for_module(
            command_name,
            module_path,
            &mut discovered,
            profile_names,
        );

        discovered
    }

    /// Returns the direct children of the root profile.
    ///
    /// The root profile (`.moldx`) owns technology profiles such as `docker`
    /// and `node`; this accessor exposes them for module resolution. It
    /// returns an empty slice when no root profile is present.
    ///
    /// # Returns
    ///
    /// The root profile's child profiles.
    pub fn profile_children(&self) -> &[Profile] {
        self.profiles
            .first()
            .map(|root| root.profiles.as_slice())
            .unwrap_or(&[])
    }

    /// Looks up a profile by name across the whole profile tree, including
    /// nested child profiles.
    ///
    /// # Arguments
    ///
    /// * `name` - The profile name to search for.
    ///
    /// # Returns
    ///
    /// The first profile matching `name`, or `None` if no profile matches.
    pub fn find_profile(&self, name: &str) -> Option<&Profile> {
        fn find<'a>(profiles: &'a [Profile], name: &str) -> Option<&'a Profile> {
            profiles.iter().find_map(|profile| {
                (profile.name == name)
                    .then_some(profile)
                    .or_else(|| find(&profile.profiles, name))
            })
        }

        find(self.profile_children(), name)
    }
}

/// Prints the configuration, executor status, profiles, and modules.
impl Display for MoldXClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn profile_count(profile: &Profile) -> usize {
            1 + profile.profiles.iter().map(profile_count).sum::<usize>()
        }

        fn write_profile(
            f: &mut fmt::Formatter<'_>,
            profile: &Profile,
            indent: usize,
        ) -> fmt::Result {
            let padding = "  ".repeat(indent);
            writeln!(f, "{}{}", padding, profile.name.bold().green())?;
            if profile.template.file_names.is_empty() {
                writeln!(f, "{}  template: empty", padding)?;
            } else {
                let template_names = profile
                    .template
                    .file_names
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(f, "{}  template: {}", padding, template_names)?;
            }
            if profile.commands.is_empty() {
                writeln!(f, "{}  commands: none", padding)?;
            } else {
                let commands = profile
                    .commands
                    .iter()
                    .map(|command| command.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(f, "{}  commands: {}", padding, commands)?;
            }
            for child in &profile.profiles {
                write_profile(f, child, indent + 1)?;
            }
            Ok(())
        }

        writeln!(f, "{}", self.config)?;
        writeln!(f, "{}", self.executor)?;
        if !self.profiles[0].commands.is_empty() {
            writeln!(f, "{}", "root commands:".bold().yellow())?;
            for command in &self.profiles[0].commands {
                writeln!(f, "  {}", command)?;
            }
        }
        let profile_total = self.profiles.first().map(profile_count).unwrap_or(0);
        writeln!(f, "{} {}", "profiles:".bold().yellow(), profile_total)?;
        for profile in &self.profiles {
            write_profile(f, profile, 1)?;
        }
        writeln!(f, "{} {}", "modules:".bold().yellow(), self.modules.len())?;
        for module in &self.modules {
            writeln!(f, "  {}", module)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_command(profile_dir: &std::path::Path, name: &str) {
        let bin = profile_dir.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(
            bin.join(format!("{}.sh", name)),
            "#!/usr/bin/env bash\nexit 0",
        )
        .unwrap();
    }

    fn make_profile_dir(root: &std::path::Path, name: &str, files: &[&str]) {
        let profile = root.join(name);
        fs::create_dir_all(profile.join("bin")).unwrap();
        let template = profile.join("template");
        fs::create_dir_all(&template).unwrap();
        for f in files {
            fs::write(template.join(f), "").unwrap();
        }
        // ensure at least one file so the template is not an empty catch-all
        // unless explicitly requested with no files
        fs::write(template.join(".keep"), "").unwrap();
    }

    fn make_client(dir: &std::path::Path) -> MoldXClient {
        let profile_root = dir.join(".moldx/profiles");
        fs::create_dir_all(&profile_root).unwrap();

        make_profile_dir(&profile_root, "docker", &["Dockerfile"]);
        write_command(&profile_root.join("docker"), "build");
        make_profile_dir(&profile_root, "node", &["package.json"]);
        write_command(&profile_root.join("node"), "test");

        let config = crate::config::MoldXConfig {
            moldx_dir: dir.join(".moldx"),
            profiles_dir: profile_root,
            profiles_dir_name: "profiles".into(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: dir.to_path_buf(),
            max_resolution_depth: 20,
        };
        MoldXClient::new(config).unwrap()
    }

    #[test]
    fn test_load_modules_discovers_matching_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let module_dir = dir.path().join("services").join("api");
        fs::create_dir_all(&module_dir).unwrap();
        fs::write(module_dir.join("Dockerfile"), "").unwrap();
        let non_module = dir.path().join("services").join("plain");
        fs::create_dir_all(&non_module).unwrap();

        let client = make_client(dir.path());
        let names: Vec<String> = client
            .modules
            .iter()
            .map(|m| m.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"api".to_string()));
        assert!(!names.contains(&"plain".to_string()));
    }

    #[test]
    fn test_resolve_modules_matches_load_modules() {
        let dir = tempfile::tempdir().unwrap();
        let module_dir = dir.path().join("worker");
        fs::create_dir_all(&module_dir).unwrap();
        fs::write(module_dir.join("package.json"), "").unwrap();

        let client = make_client(dir.path());
        let resolved = client.resolve_modules().unwrap();
        assert_eq!(resolved.len(), client.modules.len());
        assert!(
            resolved
                .iter()
                .any(|m| m.path == module_dir.canonicalize().unwrap())
        );
    }

    #[test]
    fn test_profiles_for_module_matches_template() {
        let dir = tempfile::tempdir().unwrap();
        let docker_module = dir.path().join("svc");
        fs::create_dir_all(&docker_module).unwrap();
        fs::write(docker_module.join("Dockerfile"), "").unwrap();

        let client = make_client(dir.path());
        let profiles = client.profiles_for_module(&docker_module);
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "docker");
    }

    #[test]
    fn test_profiles_for_module_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let empty_module = dir.path().join("empty");
        fs::create_dir_all(&empty_module).unwrap();

        let client = make_client(dir.path());
        let profiles = client.profiles_for_module(&empty_module);
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_commands_for_module_resolves() {
        let dir = tempfile::tempdir().unwrap();
        let docker_module = dir.path().join("svc");
        fs::create_dir_all(&docker_module).unwrap();
        fs::write(docker_module.join("Dockerfile"), "").unwrap();

        let client = make_client(dir.path());
        let commands = client.commands_for_module("build", &docker_module, &[]);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "build");
    }

    #[test]
    fn test_commands_for_module_with_profile_filter() {
        let dir = tempfile::tempdir().unwrap();
        let docker_module = dir.path().join("svc");
        fs::create_dir_all(&docker_module).unwrap();
        fs::write(docker_module.join("Dockerfile"), "").unwrap();

        let client = make_client(dir.path());
        let commands = client.commands_for_module("build", &docker_module, &["node".into()]);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_commands_for_module_unknown_command() {
        let dir = tempfile::tempdir().unwrap();
        let docker_module = dir.path().join("svc");
        fs::create_dir_all(&docker_module).unwrap();
        fs::write(docker_module.join("Dockerfile"), "").unwrap();

        let client = make_client(dir.path());
        let commands = client.commands_for_module("nope", &docker_module, &[]);
        assert!(commands.is_empty());
    }
}
