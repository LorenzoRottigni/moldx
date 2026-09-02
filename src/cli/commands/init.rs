use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::client::MoldXClient;

fn profile_dir_for_parts(client: &MoldXClient, parts: &[String]) -> PathBuf {
    let mut path = client.config.profiles_dir.clone();
    for part in parts {
        path = path.join(part);
    }
    path
}

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
pub async fn init(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        let moldx_dir: PathBuf = client.config.moldx_dir.clone();
        let bin_dir = moldx_dir.join(&client.config.bin_dir_name);
        let profiles_dir: PathBuf = client.config.profiles_dir.clone();
        let default_profile_dir = profiles_dir.join("default");
        let default_bin_dir = default_profile_dir.join(&client.config.bin_dir_name);
        let default_template_dir = default_profile_dir.join(&client.config.template_dir_name);

        fs::create_dir_all(&bin_dir)?;
        fs::create_dir_all(&default_bin_dir)?;
        fs::create_dir_all(&default_template_dir)?;
        fs::create_dir_all(&profiles_dir)?;
        fs::write(bin_dir.join(".keep"), "")?;
        fs::write(default_bin_dir.join(".keep"), "")?;
        fs::write(default_template_dir.join(".keep"), "")?;
        fs::write(profiles_dir.join(".keep"), "")?;

        let readme_path = moldx_dir.join("README.md");
        if readme_path.exists() {
            println!("README.md already exists: {}", readme_path.display());
        } else {
            let content = "# .moldx";
            fs::write(&readme_path, content)?;
            println!("Wrote {}", readme_path.display());
        }
        return Ok(());
    }

    let entity = &args[0];
    match entity.as_str() {
        "profile" => {
            let profile_parts = &args[1..];
            if profile_parts.is_empty() {
                return Ok(());
            }
            let profile_dir = profile_dir_for_parts(client, profile_parts);
            let bin_dir = profile_dir.join(&client.config.bin_dir_name);
            let template_dir = profile_dir.join(&client.config.template_dir_name);
            fs::create_dir_all(&bin_dir)?;
            fs::create_dir_all(&template_dir)?;
            fs::write(bin_dir.join(".keep"), "")?;
            fs::write(template_dir.join(".keep"), "")?;
            println!("Created profile {} at {}", profile_parts.join(" > "), profile_dir.display());
        }
        "command" => {
            let command_name = args.get(2).cloned().unwrap_or_else(|| "build".to_string());
            let profile_parts = if args.len() > 2 {
                args[1..args.len() - 1].to_vec()
            } else if args.len() == 2 {
                vec![args[1].clone()]
            } else {
                vec![]
            };
            let profile_dir = profile_dir_for_parts(client, &profile_parts);
            let script_path = profile_dir.join(&client.config.bin_dir_name).join(format!("{}.sh", command_name));
            fs::create_dir_all(script_path.parent().unwrap())?;
            fs::write(&script_path, "#!/usr/bin/env bash\nset -euo pipefail\nMODULE_PATH=\"${1:-.}\"\n")?;
            #[cfg(unix)] {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms)?;
            }
            println!("Created command {} at {}", command_name, script_path.display());
        }
        "template" => {
            let profile_parts = if args.len() > 2 {
                args[1..args.len() - 1].to_vec()
            } else {
                vec![]
            };
            let files = if args.len() > 2 {
                vec![args[args.len() - 1].clone()]
            } else {
                vec![]
            };
            let profile_dir = profile_dir_for_parts(client, &profile_parts);
            let template_dir = profile_dir.join(&client.config.template_dir_name);
            fs::create_dir_all(&template_dir)?;
            if files.is_empty() {
                fs::write(template_dir.join(".keep"), "")?;
                println!("Created template at {}", template_dir.display());
            }
            for file in files {
                let path = template_dir.join(&file);
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, "")?;
                println!("Created template file {} at {}", file, path.display());
            }
        }
        _ => {
            if !args.is_empty() {
                return Ok(());
            }
        }
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
        let result = init(&client, vec![]).await;
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/bin/.keep").exists());
        assert!(dir.path().join(".moldx/profiles/.keep").exists());
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
        let result = init(&client, vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_existing_readme() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        fs::write(dir.path().join(".moldx/README.md"), "existing").unwrap();
        let result = init(&client, vec![]).await;
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
        let result = init(&client, vec![]).await;
        assert!(result.is_ok());
        assert!(profiles_dir.exists());
    }
}
