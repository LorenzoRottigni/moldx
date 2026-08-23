//! Entry point of the MoldX command line application.
//!
//! Parses the CLI arguments, builds the project configuration and client,
//! prints a snapshot of the resolved project, and dispatches to the
//! selected subcommand.

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
pub mod profile;
pub mod tui;
pub mod types;

/// Runs the MoldX command line application.
///
/// Builds the configuration and client from the parsed CLI arguments,
/// prints a snapshot of the resolved project, and dispatches to the
/// selected subcommand.
///
/// # Errors
///
/// Returns an error if the configuration or client cannot be created, or if
/// the selected subcommand fails.
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
