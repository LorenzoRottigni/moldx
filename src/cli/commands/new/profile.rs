use crate::client::MoldXClient;
use crate::errors::MoldXError2;
use anyhow::Result;
use std::fs;

/// Scaffolds a new profile directory.
///
/// Creates the profile directory with empty bin and template directories,
/// each containing a `.keep` placeholder.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw arguments; `args[1]` is the profile name.
///
/// # Returns
///
/// Ok once the profile directory has been created.
///
/// # Errors
///
/// Returns [`MoldXError2::NewProfileUsage`] when the profile name is
/// missing, [`MoldXError2::ProfileAlreadyExists`] if the profile already
/// exists, and any IO error raised while creating directories or files.
pub fn new_profile(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let profile_name = args
        .get(1)
        .ok_or(MoldXError2::NewProfileUsage)?;
    let profile_dir = client.config.profiles_dir.join(profile_name);
    if profile_dir.exists() {
        return Err(MoldXError2::ProfileAlreadyExists { path: profile_dir }.into());
    }
    let bin_dir = profile_dir.join(&client.config.bin_dir_name);
    let template_dir = profile_dir.join(&client.config.template_dir_name);
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&template_dir)?;
    fs::write(bin_dir.join(".keep"), "")?;
    fs::write(template_dir.join(".keep"), "")?;
    println!("Created profile {} at {}", profile_name, profile_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn make_client(dir: &std::path::Path) -> MoldXClient {
        let moldx_dir = dir.join(".moldx");
        let profiles_dir = moldx_dir.join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();
        let config = crate::config::MoldXConfig {
            moldx_dir,
            profiles_dir,
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
    fn test_new_profile_success() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_profile(&client, vec!["new".into(), "myprofile".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/myprofile/bin/.keep").exists());
        assert!(dir.path().join(".moldx/profiles/myprofile/template/.keep").exists());
    }

    #[test]
    fn test_new_profile_already_exists() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        fs::create_dir(dir.path().join(".moldx/profiles/myprofile")).unwrap();
        let result = new_profile(&client, vec!["new".into(), "myprofile".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_profile_missing_name() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_profile(&client, vec!["new".into()]);
        assert!(result.is_err());
    }
}
