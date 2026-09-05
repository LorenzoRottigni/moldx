//! Discovered project modules.
//!
//! [`crate::module::Module`] represents any directory under the modules root whose file
//! names satisfy at least one profile's template.

use anyhow::{Result, bail};
use owo_colors::OwoColorize;
use std::fmt::{self, Display};
use std::path::PathBuf;

use crate::errors::MoldXError2;
use crate::fs::resolve_name;
use crate::profile::Profile;
use crate::types::Entity;

/// A directory on the filesystem that matches one or more profiles.
///
/// A `Module` is any subdirectory of the modules root whose file names
/// satisfy at least one profile's template. The `profiles` field stores
/// indices into the resolved profile list.
#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub profiles: Vec<usize>,
}

impl Module {
    /// Creates a module with an explicit set of matching profile indices.
    ///
    /// # Arguments
    ///
    /// * `path` - The module directory.
    /// * `profiles` - Indices of profiles whose templates match.
    ///
    /// # Returns
    ///
    /// The new module.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a directory or its name cannot
    /// be resolved.
    pub fn new(path: PathBuf, profiles: Vec<usize>) -> Result<Self> {
        if !path.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: path.to_path_buf(),
                kind: "module",
            });
        }
        Ok(Self {
            name: resolve_name(&path, Entity::Module)?,
            path,
            profiles,
        })
    }

    /// Resolves a module by matching its directory against all known profiles.
    ///
    /// Only profiles whose templates match the directory's file names are
    /// included in the returned module.
    ///
    /// # Arguments
    ///
    /// * `path` - The module directory.
    /// * `profiles` - The full list of resolved profiles.
    ///
    /// # Returns
    ///
    /// The resolved module with its matching profile indices.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a directory or its name cannot
    /// be resolved.
    pub fn resolve(path: PathBuf, profiles: &[Profile]) -> Result<Self> {
        if !path.is_dir() {
            bail!(MoldXError2::PathNotFound {
                path: path.clone(),
                kind: "module",
            });
        }

        let name = resolve_name(&path, Entity::Module)?;

        let matching_profiles = profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| profile.template.matches(&path))
            .map(|(index, _)| index)
            .collect();

        Ok(Self {
            name,
            path,
            profiles: matching_profiles,
        })
    }
}

/// Prints the module name, matching profile count, and path.
impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let profile_count = self.profiles.len();
        let profile_label = match profile_count {
            0 => "no profiles".dimmed().to_string(),
            1 => "1 profile".cyan().to_string(),
            n => format!("{} profiles", n).cyan().to_string(),
        };

        write!(
            f,
            "{} [{}] {}",
            self.name.bold().green(),
            profile_label,
            format!("@ {}", self.path.display()).dimmed()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_module_new_valid() {
        let dir = tempdir().unwrap();
        let module_dir = dir.path().join("my-module");
        fs::create_dir(&module_dir).unwrap();
        let m = Module::new(module_dir.clone(), vec![0, 2]).unwrap();
        assert_eq!(m.name, "my-module");
        assert_eq!(m.path, module_dir);
        assert_eq!(m.profiles, vec![0, 2]);
    }

    #[test]
    fn test_module_new_no_filename() {
        let result = Module::new(PathBuf::from("/"), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_module_display_zero_profiles() {
        let dir = tempdir().unwrap();
        let m = Module::new(dir.path().to_path_buf(), vec![]).unwrap();
        let display = m.to_string();
        assert!(display.contains("no profiles"));
    }

    #[test]
    fn test_module_display_one_profile() {
        let dir = tempdir().unwrap();
        let m = Module::new(dir.path().to_path_buf(), vec![0]).unwrap();
        let display = m.to_string();
        assert!(display.contains("1 profile"));
    }

    #[test]
    fn test_module_display_multiple_profiles() {
        let dir = tempdir().unwrap();
        let m = Module::new(dir.path().to_path_buf(), vec![0, 1, 2]).unwrap();
        let display = m.to_string();
        assert!(display.contains("3 profiles"));
    }
}
