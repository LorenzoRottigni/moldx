use crate::v2::client::MoldXClient;
use anyhow::{Result, bail};
use std::fs;

pub fn new_strategy(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let strategy_name = args
        .get(1)
        .expect("Usage: moldx new strategy <strategy>");
    let strategy_dir = client.config.strategies_dir.join(strategy_name);
    if strategy_dir.exists() {
        bail!("Strategy already exists: {}", strategy_dir.display());
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