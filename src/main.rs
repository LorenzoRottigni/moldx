use anyhow::Result;
use clap::Parser;

pub mod cli;
pub mod client;
pub mod command;
pub mod config;
pub mod constants;
pub mod errors;
pub mod executor;
pub mod fs;
pub mod module;
pub mod strategy;
pub mod template;
pub mod tui;
pub mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = config::MoldXConfig::new(
        cli.moldx_dir.clone(),
        cli.strategies_dir_name.clone(),
        cli.bin_dir_name.clone(),
        cli.template_dir_name.clone(),
        cli.templates_dir_name.clone(),
        cli.max_resolution_depth.clone(),
        cli.modules_dir.clone(),
    )?;
    let client = client::MoldXClient::new(config)?;
    print!("{}", client);
    cli.exec_with(client).await?;
    Ok(())
}
