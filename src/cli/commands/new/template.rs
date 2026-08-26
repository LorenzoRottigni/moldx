use crate::client::MoldXClient;
use crate::errors::MoldXError2;
use anyhow::Result;
use std::fs;

/// Scaffolds a new template directory for a profile.
///
/// Accepts either `<template>` (defaulting to the `default` profile) or
/// `<profile> <template>`. The created template contains a `.keep`
/// placeholder.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw arguments; see above for the accepted forms.
///
/// # Returns
///
/// Ok once the template directory has been created.
///
/// # Errors
///
/// Returns [`MoldXError2::NewUsage`] on malformed arguments,
/// [`MoldXError2::ProfileNotFound`] if the profile does not exist, and any
/// IO error raised while creating directories or files.
pub fn new_template(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let (profile_name, template_name) = match args.len() {
        2 => ("default".to_string(), args[1].clone()),
        3 => (args[1].clone(), args[2].clone()),
        _ => return Err(MoldXError2::NewUsage.into()),
    };
    let profile_dir = client.config.profiles_dir.join(&profile_name);
    if !profile_dir.exists() {
        return Err(MoldXError2::ProfileNotFound { name: profile_name }.into());
    }
    let template_dir = profile_dir.join(&client.config.templates_dir_name).join(&template_name);
    fs::create_dir_all(&template_dir)?;
    fs::write(template_dir.join(".keep"), "")?;
    println!(
        "Created template {} for profile {} at {}",
        template_name,
        profile_name,
        template_dir.display()
    );
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

    fn create_profile(dir: &std::path::Path, name: &str) {
        let profile_dir = dir.join(".moldx/profiles").join(name);
        fs::create_dir_all(profile_dir.join("bin")).unwrap();
        fs::create_dir_all(profile_dir.join("template")).unwrap();
    }

    #[test]
    fn test_new_template_default_profile() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "default");
        let client = make_client(dir.path());
        let result = new_template(&client, vec!["template".into(), "mytpl".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/default/templates/mytpl/.keep").exists());
    }

    #[test]
    fn test_new_template_explicit_profile() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "docker");
        let client = make_client(dir.path());
        let result = new_template(&client, vec!["template".into(), "docker".into(), "mytpl".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/docker/templates/mytpl/.keep").exists());
    }

    #[test]
    fn test_new_template_profile_not_found() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_template(&client, vec!["template".into(), "nonexistent".into(), "mytpl".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_template_wrong_arg_count() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_template(&client, vec!["template".into()]);
        assert!(result.is_err());
    }
}
