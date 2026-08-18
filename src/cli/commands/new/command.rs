use crate::client::MoldXClient;
use crate::errors::MoldXError;
use anyhow::Result;
use std::{fs, io::Write};

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