mod command;
mod module;
mod profile;
mod template;

use crate::{client::MoldXClient, types::Entity, errors::MoldXError2};
use anyhow::Result;

/// Scaffolds a new MoldX entity.
///
/// Dispatches to the entity-specific scaffold handler based on the first
/// argument, which must parse as an [`Entity`].
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw arguments; the first selects the entity kind.
///
/// # Returns
///
/// Ok once the entity has been scaffolded.
///
/// # Errors
///
/// Returns [`MoldXError2::NewUsage`] when no arguments are given, and any
/// error raised while parsing the entity or scaffolding it.
pub async fn new(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(MoldXError2::NewUsage.into());
    }

    let entity = args[0]
        .parse::<Entity>()?;

    match entity {
        Entity::Profile => profile::new_profile(client, args)?,
        Entity::Template => template::new_template(client, args)?,
        Entity::Module => module::new_module(client, args)?,
        Entity::Command => command::new_command(client, args)?
    }

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

    #[tokio::test]
    async fn test_new_empty_args() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new(&client, vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_invalid_entity() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new(&client, vec!["invalid".into(), "name".into()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new(&client, vec!["profile".into(), "myprofile".into()]).await;
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/myprofile").exists());
    }

    #[tokio::test]
    async fn test_new_template() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "default");
        let client = make_client(dir.path());
        let result = new(&client, vec!["template".into(), "mytpl".into()]).await;
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/default/templates/mytpl").exists());
    }

    #[tokio::test]
    async fn test_new_command() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "default");
        let client = make_client(dir.path());
        let result = new(&client, vec!["command".into(), "build".into()]).await;
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/default/bin/build.sh").exists());
    }

    #[tokio::test]
    async fn test_new_module() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let module_path = dir.path().join("new-module");
        let result = new(&client, vec!["module".into(), module_path.to_str().unwrap().into()]).await;
        assert!(result.is_ok());
        assert!(module_path.exists());
    }
}
