use crate::client::MoldXClient;
use crate::errors::MoldXError;
use anyhow::Result;
use std::fs;

/// Scaffolds a new template directory for a strategy.
///
/// Accepts either `<template>` (defaulting to the `default` strategy) or
/// `<strategy> <template>`. The created template contains a `.keep`
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
/// Returns [`MoldXError::NewTemplateUsage`] on malformed arguments,
/// [`MoldXError::StrategyNotFound`] if the strategy does not exist, and any
/// IO error raised while creating directories or files.
pub fn new_template(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let (strategy_name, template_name) = match args.len() {
        2 => ("default".to_string(), args[1].clone()),
        3 => (args[1].clone(), args[2].clone()),
        _ => return Err(MoldXError::NewTemplateUsage.into()),
    };
    let strategy_dir = client.config.strategies_dir.join(&strategy_name);
    if !strategy_dir.exists() {
        return Err(MoldXError::StrategyNotFound { name: strategy_name }.into());
    }
    let template_dir = strategy_dir.join(&client.config.templates_dir_name).join(&template_name);
    fs::create_dir_all(&template_dir)?;
    fs::write(template_dir.join(".keep"), "")?;
    println!(
        "Created template {} for strategy {} at {}",
        template_name,
        strategy_name,
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
    fn test_new_template_default_strategy() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".moldx/strategies/default")).unwrap();
        let client = make_client(dir.path());
        let result = new_template(&client, vec!["template".into(), "mytpl".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/strategies/default/templates/mytpl/.keep").exists());
    }

    #[test]
    fn test_new_template_explicit_strategy() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".moldx/strategies/docker")).unwrap();
        let client = make_client(dir.path());
        let result = new_template(&client, vec!["template".into(), "docker".into(), "mytpl".into()]);
        assert!(result.is_ok());
        assert!(dir.path().join(".moldx/strategies/docker/templates/mytpl/.keep").exists());
    }

    #[test]
    fn test_new_template_strategy_not_found() {
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