use crate::client::MoldXClient;
use crate::errors::MoldXError;
use anyhow::Result;
use std::fs;

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