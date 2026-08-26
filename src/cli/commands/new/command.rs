use crate::client::MoldXClient;
use crate::errors::MoldXError2;
use anyhow::Result;
use std::{fs, io::Write};

/// Scaffolds a new command script in a profile's bin directory.
///
/// Accepts either `<command>` (defaulting to the `default` profile) or
/// `<profile> <command>`. The generated script is executable and receives
/// the module path as its first argument.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw arguments; see above for the accepted forms.
///
/// # Returns
///
/// Ok once the command script has been created.
///
/// # Errors
///
/// Returns [`MoldXError2::NewUsage`] on malformed arguments,
/// [`MoldXError2::ProfileNotFound`] if the profile does not exist,
/// [`MoldXError2::CommandAlreadyExists`] if the script already exists, and
/// any IO error raised while writing the script.
pub fn new_command(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let (profile_name, command_name) = match args.len() {
        2 => ("default".to_string(), args[1].clone()),
        3 => (args[1].clone(), args[2].clone()),
        _ => return Err(MoldXError2::NewUsage.into()),
    };
    let profile_dir = client.config.profiles_dir.join(&profile_name);
    if !profile_dir.exists() {
        return Err(MoldXError2::ProfileNotFound { name: profile_name }.into());
    }
    let bin_dir = profile_dir.join(&client.config.bin_dir_name);
    fs::create_dir_all(&bin_dir)?;
    let script_path = bin_dir.join(format!("{}.sh", command_name));
    if script_path.exists() {
        return Err(MoldXError2::CommandAlreadyExists { path: script_path }.into());
    }
    let mut file = fs::File::create(&script_path)?;
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nMODULE_PATH=\"$1\"\nprintf '[moldx] {} {}\\n'\n",
        profile_name, command_name
    );
    file.write_all(script.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }
    println!("Created command {} at {}", command_name, script_path.display());
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
    fn test_new_command_default_profile() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "default");
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "build".into()]);
        assert!(result.is_ok());
        let script = dir.path().join(".moldx/profiles/default/bin/build.sh");
        assert!(script.exists());
        let content = fs::read_to_string(&script).unwrap();
        assert!(content.contains("build"));
    }

    #[test]
    fn test_new_command_explicit_profile() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "docker");
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "docker".into(), "deploy".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/profiles/docker/bin/deploy.sh").exists());
    }

    #[test]
    fn test_new_command_profile_not_found() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "nonexistent".into(), "build".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_command_already_exists() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "default");
        fs::write(dir.path().join(".moldx/profiles/default/bin/build.sh"), "").unwrap();
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "build".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_command_wrong_arg_count() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into()]);
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_new_command_executable_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "default");
        let client = make_client(dir.path());
        new_command(&client, vec!["command".into(), "test".into()]).unwrap();
        let script = dir.path().join(".moldx/profiles/default/bin/test.sh");
        let perms = fs::metadata(&script).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }
}
