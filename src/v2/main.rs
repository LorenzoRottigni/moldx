use anyhow::{Result};
use clap::Parser;
use crate::v2::config::{MoldXConfig};
use crate::v2::client::{MoldXClient};
use crate::v2::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = MoldXConfig::new(
        cli.moldx_dir.clone(),
        cli.strategies_dir_name.clone(),
        cli.bin_dir_name.clone(),
        cli.template_dir_name.clone(),
        cli.templates_dir_name.clone(),
    );
    let client = MoldXClient::new(config)?;
    cli.exec_with(client).await?;
    Ok(())
}
