use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::client::MoldXClient;
use crate::errors::MoldXError2;

fn profile_dir_for_parts(client: &MoldXClient, parts: &[String]) -> PathBuf {
    let mut path = client.config.profiles_dir.clone();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            path = path.join(&client.config.profiles_dir_name);
        }
        path = path.join(part);
    }
    path
}

fn default_profile_parts() -> Vec<String> {
    vec!["default".to_string()]
}

fn profile_parts_for_template(client: &MoldXClient, args: &[String]) -> (Vec<String>, Vec<String>) {
    fn profile_exists(profiles: &[crate::profile::Profile], parts: &[String]) -> bool {
        let Some((head, tail)) = parts.split_first() else {
            return true;
        };
        profiles
            .iter()
            .any(|profile| profile.name == *head && profile_exists(&profile.profiles, tail))
    }

    let mut best = 0;
    for length in 1..=args.len() {
        if profile_exists(client.profile_children(), &args[..length])
            || profile_dir_for_parts(client, &args[..length]).is_dir()
        {
            best = length;
        }
    }

    if best == 0 {
        (default_profile_parts(), args.to_vec())
    } else {
        (args[..best].to_vec(), args[best..].to_vec())
    }
}

/// Initializes a new `.moldx` directory structure or scaffolds entities.
///
/// The following forms are supported (matching the README):
///
/// - `moldx init` -> creates `.moldx/README.md`, `.moldx/bin/.keep`, and
///   `.moldx/profiles/.keep` plus a scaffolded `default` profile.
/// - `moldx init profile <profile...>` -> creates a profile (with nested
///   support) containing `bin` and `template` directories.
/// - `moldx init command [profile...] <command>` -> creates an executable
///   command script in the profile's `bin` directory (with nested support).
/// - `moldx init template [profile...] [file...]` -> creates template files
///   in the profile's `template` directory (with nested support).
///
/// # Arguments
///
/// * `client` - The initialized MoldX client providing the configuration.
/// * `args` - Raw arguments; the first selects the entity when present.
///
/// # Returns
///
/// Ok once the requested entity has been created.
///
/// # Errors
///
/// Returns an error for missing or unknown entities, or if directories or
/// files cannot be created.
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
        "profile" => init_profile(client, &args[1..]),
        "command" => init_command(client, &args[1..]),
        "template" => init_template(client, &args[1..]),
        other => Err(MoldXError2::UnknownInitEntity {
            entity: other.to_string(),
        }
        .into()),
    }
}

/// Creates a profile (optionally nested) with `bin` and `template` dirs.
fn init_profile(client: &MoldXClient, parts: &[String]) -> Result<()> {
    if parts.is_empty() {
        return Err(MoldXError2::InitProfileUsage.into());
    }
    let profile_dir = profile_dir_for_parts(client, parts);
    let bin_dir = profile_dir.join(&client.config.bin_dir_name);
    let template_dir = profile_dir.join(&client.config.template_dir_name);
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&template_dir)?;
    if parts.len() > 1 {
        let parent_template = profile_dir_for_parts(client, &parts[..parts.len() - 1])
            .join(&client.config.template_dir_name);
        if parent_template.is_dir() {
            for entry in walkdir::WalkDir::new(&parent_template)
                .min_depth(1)
                .into_iter()
                .flatten()
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let relative = entry.path().strip_prefix(&parent_template).map_err(|_| {
                    MoldXError2::PathNotFound {
                        path: entry.path().to_path_buf(),
                        kind: "template",
                    }
                })?;
                if relative.to_string_lossy().starts_with('.') {
                    continue;
                }
                let child_file = template_dir.join(relative);
                if let Some(parent) = child_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), child_file)?;
            }
        }
    }
    fs::write(bin_dir.join(".keep"), "")?;
    fs::write(template_dir.join(".keep"), "")?;
    println!(
        "Created profile {} at {}",
        parts.join(" > "),
        profile_dir.display()
    );
    Ok(())
}

/// Creates an executable command script `[profile...] <command>`.
///
/// When no profile is given, the `default` profile is used. The last
/// argument is treated as the command name; all preceding arguments form the
/// (possibly nested) profile hierarchy.
fn init_command(client: &MoldXClient, sub: &[String]) -> Result<()> {
    if sub.is_empty() {
        return Err(MoldXError2::InitCommandUsage.into());
    }

    let command_name = sub.last().expect("non-empty sub").clone();
    let profile_parts = if sub.len() == 1 {
        default_profile_parts()
    } else {
        sub[..sub.len() - 1].to_vec()
    };

    let profile_dir = profile_dir_for_parts(client, &profile_parts);
    let script_path = profile_dir
        .join(&client.config.bin_dir_name)
        .join(format!("{}.sh", command_name));
    fs::create_dir_all(script_path.parent().expect("script has a parent"))?;
    fs::write(
        &script_path,
        "#!/usr/bin/env bash\nset -euo pipefail\nMODULE_PATH=\"${1:-.}\"\nprintf '[moldx] {} {}\\n'\n",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }
    println!(
        "Created command {} at {}",
        command_name,
        script_path.display()
    );
    Ok(())
}

/// Creates template files `[profile...] [file...]`.
///
/// When no profile is given, the `default` profile is used. The longest
/// existing profile hierarchy is recognized as the leading arguments and all
/// remaining arguments are created as template files.
fn init_template(client: &MoldXClient, sub: &[String]) -> Result<()> {
    if sub.is_empty() {
        return Err(MoldXError2::InitTemplateUsage.into());
    }

    let (profile_parts, files) = profile_parts_for_template(client, sub);

    let profile_dir = profile_dir_for_parts(client, &profile_parts);
    let template_dir = profile_dir.join(&client.config.template_dir_name);
    fs::create_dir_all(&template_dir)?;

    for file in files {
        let path = template_dir.join(&file);
        fs::create_dir_all(path.parent().expect("template file has a parent"))?;
        fs::write(&path, "")?;
        println!("Created template file {} at {}", file, path.display());
    }
    if fs::read_dir(&template_dir)?.next().is_none() {
        fs::write(template_dir.join(".keep"), "")?;
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
        assert!(
            dir.path()
                .join(".moldx/profiles/default/bin/.keep")
                .exists()
        );
        assert!(
            dir.path()
                .join(".moldx/profiles/default/template/.keep")
                .exists()
        );
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

    #[tokio::test]
    async fn test_init_profile_single() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client, vec!["profile".into(), "docker".into()]).await;
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/docker/bin/.keep").exists());
        assert!(
            dir.path()
                .join(".moldx/profiles/docker/template/.keep")
                .exists()
        );
    }

    #[tokio::test]
    async fn test_init_profile_nested() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(
            &client,
            vec!["profile".into(), "node".into(), "nuxt".into()],
        )
        .await;
        assert!(result.is_ok());
        assert!(
            dir.path()
                .join(".moldx/profiles/node/profiles/nuxt/bin/.keep")
                .exists()
        );
        assert!(
            dir.path()
                .join(".moldx/profiles/node/profiles/nuxt/template/.keep")
                .exists()
        );
    }

    #[tokio::test]
    async fn test_init_profile_missing_name_errors() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client, vec!["profile".into()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_init_command_explicit_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(
            &client,
            vec!["command".into(), "docker".into(), "build".into()],
        )
        .await;
        assert!(result.is_ok());
        let script = dir.path().join(".moldx/profiles/docker/bin/build.sh");
        assert!(script.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&script).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o755);
        }
    }

    #[tokio::test]
    async fn test_init_command_nested_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(
            &client,
            vec!["command".into(), "node".into(), "nuxt".into(), "dev".into()],
        )
        .await;
        assert!(result.is_ok());
        let script = dir
            .path()
            .join(".moldx/profiles/node/profiles/nuxt/bin/dev.sh");
        assert!(script.exists());
    }

    #[tokio::test]
    async fn test_init_command_default_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client, vec!["command".into(), "build".into()]).await;
        assert!(result.is_ok());
        let script = dir.path().join(".moldx/profiles/default/bin/build.sh");
        assert!(script.exists());
    }

    #[tokio::test]
    async fn test_init_command_missing_name_errors() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client, vec!["command".into()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_init_template_with_profile_and_file() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        init(&client, vec!["profile".into(), "docker".into()])
            .await
            .unwrap();
        let result = init(
            &client,
            vec!["template".into(), "docker".into(), "Dockerfile".into()],
        )
        .await;
        assert!(result.is_ok());
        assert!(
            dir.path()
                .join(".moldx/profiles/docker/template/Dockerfile")
                .exists()
        );
    }

    #[tokio::test]
    async fn test_init_template_nested_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        init(&client, vec!["profile".into(), "node".into()])
            .await
            .unwrap();
        init(
            &client,
            vec!["profile".into(), "node".into(), "nuxt".into()],
        )
        .await
        .unwrap();
        let result = init(
            &client,
            vec![
                "template".into(),
                "node".into(),
                "nuxt".into(),
                "nuxt.config.ts".into(),
            ],
        )
        .await;
        assert!(result.is_ok());
        assert!(
            dir.path()
                .join(".moldx/profiles/node/profiles/nuxt/template/nuxt.config.ts")
                .exists()
        );
    }

    #[tokio::test]
    async fn test_init_template_default_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client, vec!["template".into(), "package.json".into()]).await;
        assert!(result.is_ok());
        assert!(
            dir.path()
                .join(".moldx/profiles/default/template/package.json")
                .exists()
        );
    }

    #[tokio::test]
    async fn test_init_unknown_entity_errors() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = init(&client, vec!["bogus".into(), "x".into()]).await;
        assert!(result.is_err());
    }
}
