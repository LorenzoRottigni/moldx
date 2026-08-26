use anyhow::Result;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use crate::client::MoldXClient;

/// Initializes a new `.moldx` directory structure.
///
/// Creates the profiles directory if missing, scaffolds a default profile
/// with empty bin and template directories, and writes a README.md unless it
/// already exists.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client providing the configuration.
///
/// # Returns
///
/// Ok once the directory structure is in place.
///
/// # Errors
///
/// Returns an error if directories or files cannot be created.
pub async fn init(
    client: &MoldXClient,
) -> Result<()> {
    let moldx_dir: PathBuf = client.config.moldx_dir.clone();
    let profiles_dir: PathBuf = client.config.profiles_dir.clone();
    let bin_dir_name: String = client.config.bin_dir_name.clone();
    let template_dir_name: String = client.config.template_dir_name.clone();

    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir)?;
        println!("Created {}", profiles_dir.display());
    } else {
        println!("Directory already exists: {}", profiles_dir.display());
    }

    let default_profile_dir = profiles_dir.join("default");
    [
        default_profile_dir.join(&bin_dir_name).join(".keep"),
        default_profile_dir.join(&template_dir_name).join(".keep"),
    ]
        .iter()
        .try_for_each(|path| {
            fs::create_dir_all(path.parent().unwrap())?;
            File::create(path)?;
            Ok::<(), std::io::Error>(())
        })?;

    let readme_path = moldx_dir.join("README.md");
    if readme_path.exists() {
        println!("README.md already exists: {}", readme_path.display());
    } else {
        let content = "# .moldx";
        fs::write(&readme_path, content)?;
        println!("Wrote {}", readme_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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

    #[tokio::test]
    async fn test_init_creates_directories_and_readme() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client).await;
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/default/bin/.keep").exists());
        assert!(dir.path().join(".moldx/profiles/default/template/.keep").exists());
        assert!(dir.path().join(".moldx/README.md").exists());
        let readme = fs::read_to_string(dir.path().join(".moldx/README.md")).unwrap();
        assert_eq!(readme, "# .moldx");
    }

    #[tokio::test]
    async fn test_init_existing_profiles_dir() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        fs::create_dir_all(client.config.profiles_dir.clone()).unwrap();
        let result = init(&client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_existing_readme() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        fs::write(dir.path().join(".moldx/README.md"), "existing").unwrap();
        let result = init(&client).await;
        assert!(result.is_ok());
        let readme = fs::read_to_string(dir.path().join(".moldx/README.md")).unwrap();
        assert_eq!(readme, "existing");
    }

    #[tokio::test]
    async fn test_init_profiles_dir_not_existing() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let profiles_dir = client.config.profiles_dir.clone();
        fs::remove_dir_all(&profiles_dir).unwrap();
        let result = init(&client).await;
        assert!(result.is_ok());
        assert!(profiles_dir.exists());
    }
}
