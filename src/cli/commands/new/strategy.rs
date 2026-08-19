use crate::client::MoldXClient;
use crate::errors::MoldXError;
use anyhow::Result;
use std::fs;

pub fn new_strategy(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let strategy_name = args
        .get(1)
        .ok_or(MoldXError::NewStrategyUsage)?;
    let strategy_dir = client.config.strategies_dir.join(strategy_name);
    if strategy_dir.exists() {
        return Err(MoldXError::StrategyAlreadyExists { path: strategy_dir }.into());
    }
    let bin_dir = strategy_dir.join(&client.config.bin_dir_name);
    let template_dir = strategy_dir.join(&client.config.template_dir_name);
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&template_dir)?;
    fs::write(bin_dir.join(".keep"), "")?;
    fs::write(template_dir.join(".keep"), "")?;
    println!("Created strategy {} at {}", strategy_name, strategy_dir.display());
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
    fn test_new_strategy_success() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_strategy(&client, vec!["new".into(), "mystrat".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/strategies/mystrat/bin/.keep").exists());
        assert!(dir.path().join(".moldx/strategies/mystrat/template/.keep").exists());
    }

    #[test]
    fn test_new_strategy_already_exists() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        fs::create_dir(dir.path().join(".moldx/strategies/mystrat")).unwrap();
        let result = new_strategy(&client, vec!["new".into(), "mystrat".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_strategy_missing_name() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_strategy(&client, vec!["new".into()]);
        assert!(result.is_err());
    }
}