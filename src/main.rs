//! moldx — technology-agnostic orchestration engine.
//!
//! Entry point: parses CLI arguments and dispatches to the appropriate handler.
//! All subcommands resolve a [`config::MoldxConfig`] before doing any work so
//! that the `.moldx/` directory is guaranteed to exist.
mod cli;
mod config;
mod executor;
mod probe;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::commands;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let moldx_dir_override = cli.moldx_dir.as_deref().map(std::path::Path::new);
    let strategies_dir_override_value = cli
        .strategies_dir
        .or_else(|| std::env::var("MOLDX_BIN_DIR").ok());
    let strategies_dir_override = strategies_dir_override_value
        .as_deref()
        .map(std::path::Path::new);

    match cli.command.unwrap_or(Commands::Ui) {
        // moldx [ui]
        Commands::Ui => {
            commands::ui::ui(moldx_dir_override, strategies_dir_override).await?;
        }

        // moldx detect <path>
        Commands::Detect { path } => {
            commands::detect::detect(path, moldx_dir_override, strategies_dir_override).await?;
        }

        // moldx list [<path>] [--depth <depth>]
        Commands::List { path, depth } => {
            commands::list::list(path, depth, moldx_dir_override, strategies_dir_override).await?;
        }

        // moldx new module ...
        Commands::New { args } => {
            commands::new::new(args, moldx_dir_override, strategies_dir_override).await?;
        }

        // moldx [strategy] <command> <path>
        Commands::Run(args) => {
            commands::run::run(args, moldx_dir_override, strategies_dir_override).await?;
        }
    }

    Ok(())
}
