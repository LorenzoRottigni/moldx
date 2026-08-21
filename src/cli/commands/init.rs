use anyhow::Result;
use std::fs;
use std::fs::File;
use std::path::{PathBuf};

use crate::client::MoldXClient;

/// Initializes a new `.moldx` directory structure.
///
/// Creates the strategies directory if missing, scaffolds a default strategy
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
    client: &MoldXClient
) -> Result<()> {
    let moldx_dir: PathBuf = client.config.moldx_dir.clone();
    let strategies_dir: PathBuf = client.config.strategies_dir.clone();
    let bin_dir_name: String = client.config.bin_dir_name.clone();
    let template_dir_name: String = client.config.bin_dir_name.clone();

    if !strategies_dir.exists() {
        fs::create_dir_all(&strategies_dir)?;
        println!("Created {}", strategies_dir.display());
    } else {
        println!("Directory already exists: {}", strategies_dir.display());
    }

    
    let default_strategy_dir = strategies_dir.join("default");
    [
        default_strategy_dir.join(&bin_dir_name).join(".keep"),
        default_strategy_dir.join(&template_dir_name).join(".keep"),
    ]
        .iter()
        .try_for_each(|path| {
            fs::create_dir_all(path.parent().unwrap())?;
            File::create(path)?;
            Ok::<(), std::io::Error>(())
        })?;

    // Write .moldx/README.md
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
        let strategies_dir = moldx_dir.join("strategies");
        fs::create_dir_all(&strategies_dir).unwrap();
        let config = crate::config::MoldXConfig {
            moldx_dir,
            strategies_dir,
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
        assert!(dir.path().join(".moldx/strategies/default/bin/.keep").exists());
        assert!(dir.path().join(".moldx/strategies/default/bin/.keep").exists());
        assert!(dir.path().join(".moldx/README.md").exists());
        let readme = fs::read_to_string(dir.path().join(".moldx/README.md")).unwrap();
        assert_eq!(readme, "# .moldx");
    }

    #[tokio::test]
    async fn test_init_existing_strategies_dir() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        fs::create_dir_all(client.config.strategies_dir.clone()).unwrap();
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
    async fn test_init_strategies_dir_not_existing() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let strategies_dir = client.config.strategies_dir.clone();
        fs::remove_dir_all(&strategies_dir).unwrap();
        let result = init(&client).await;
        assert!(result.is_ok());
        assert!(strategies_dir.exists());
    }
}
