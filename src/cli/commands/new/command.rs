use crate::client::MoldXClient;
use crate::errors::MoldXError;
use anyhow::Result;
use std::{fs, io::Write};

/// Scaffolds a new command script in a strategy's bin directory.
///
/// Accepts either `<command>` (defaulting to the `default` strategy) or
/// `<strategy> <command>`. The generated script is executable and receives
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
/// Returns [`MoldXError::NewCommandUsage`] on malformed arguments,
/// [`MoldXError::StrategyNotFound`] if the strategy does not exist,
/// [`MoldXError::CommandAlreadyExists`] if the script already exists, and
/// any IO error raised while writing the script.
pub fn new_command(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let (strategy_name, command_name) = match args.len() {
        2 => ("default".to_string(), args[1].clone()),
        3 => (args[1].clone(), args[2].clone()),
        _ => return Err(MoldXError::NewCommandUsage.into()),
    };
    let strategy_dir = client.config.strategies_dir.join(&strategy_name);
    if !strategy_dir.exists() {
        return Err(MoldXError::StrategyNotFound { name: strategy_name }.into());
    }
    let bin_dir = strategy_dir.join(&client.config.bin_dir_name);
    fs::create_dir_all(&bin_dir)?;
    let script_path = bin_dir.join(format!("{}.sh", command_name));
    if script_path.exists() {
        return Err(MoldXError::CommandAlreadyExists { path: script_path }.into());
    }
    let mut file = fs::File::create(&script_path)?;
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nMODULE_PATH=\"$1\"\nprintf '[moldx] {} {}\\n'\n",
        strategy_name, command_name
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

    #[test]
    fn test_new_command_default_strategy() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".moldx/strategies/default/bin")).unwrap();
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "build".into()]);
        assert!(result.is_ok());
        let script = dir.path().join(".moldx/strategies/default/bin/build.sh");
        assert!(script.exists());
        let content = fs::read_to_string(&script).unwrap();
        assert!(content.contains("build"));
    }

    #[test]
    fn test_new_command_explicit_strategy() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".moldx/strategies/docker/bin")).unwrap();
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "docker".into(), "deploy".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/strategies/docker/bin/deploy.sh").exists());
    }

    #[test]
    fn test_new_command_strategy_not_found() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_command(&client, vec!["command".into(), "nonexistent".into(), "build".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_command_already_exists() {
        let dir = tempdir().unwrap();
        let bin_dir = dir.path().join(".moldx/strategies/default/bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("build.sh"), "").unwrap();
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
        fs::create_dir_all(dir.path().join(".moldx/strategies/default/bin")).unwrap();
        let client = make_client(dir.path());
        new_command(&client, vec!["command".into(), "test".into()]).unwrap();
        let script = dir.path().join(".moldx/strategies/default/bin/test.sh");
        let perms = fs::metadata(&script).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }
}