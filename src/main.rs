use anyhow::{Result};
use clap::Parser;

pub mod client;
pub mod strategy;
pub mod fs;
pub mod template;
pub mod command;
pub mod config;
pub mod cli;
pub mod errors;
pub mod executor;
pub mod module;
pub mod types;
pub mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = config::MoldXConfig::new(
        cli.moldx_dir.clone(),
        cli.strategies_dir_name.clone(),
        cli.bin_dir_name.clone(),
        cli.template_dir_name.clone(),
        cli.templates_dir_name.clone(),
        cli.modules_dir.clone(),
    )?;
    let client = client::MoldXClient::new(config)?;
    print!("{}", client);
    cli.exec_with(client).await?;
    Ok(())
}
