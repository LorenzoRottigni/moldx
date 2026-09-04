//! Entry point of the MoldX command line application.
//!
//! Parses the CLI arguments, builds the project configuration and client,
//! prints a snapshot of the resolved project, and dispatches to the
//! selected subcommand.

use anyhow::Result;
use clap::Parser;

/// Command line argument parsing and subcommand dispatch.
pub mod cli;
/// Client facade that ties together profiles, modules, and execution.
pub mod client;
/// Executable scripts that live inside a profile's bin directory.
pub mod command;
/// Configuration resolution for a `.moldx` project tree.
pub mod config;
/// Default directory and file names used by MoldX.
pub mod constants;
/// Error types raised by MoldX operations.
pub mod errors;
/// Process spawning, tracking, and lifecycle management.
pub mod executor;
/// Filesystem helpers for reading directories and discovering paths.
pub mod fs;
/// A directory on the filesystem that matches one or more profiles.
pub mod module;
/// A named collection of templates, commands, and nested profiles.
pub mod profile;
/// Template files used to identify and scaffold modules.
pub mod template;
/// Interactive terminal UI built with ratatui.
pub mod tui;
/// Shared type definitions (e.g. [`Entity`]).
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
    // Scaffolding commands (init/new) may create `.moldx` and therefore must
    // not fail when the directory does not yet exist.
    let create_if_missing = matches!(
        &cli.command,
        Some(cli::Command::Init { .. }) | Some(cli::Command::New { .. })
    );
    let config = config::MoldXConfig::new(
        cli.moldx_dir.clone(),
        cli.profiles_dir_name.clone(),
        cli.bin_dir_name.clone(),
        cli.template_dir_name.clone(),
        cli.templates_dir_name.clone(),
        cli.max_resolution_depth,
        cli.modules_dir.clone(),
        create_if_missing,
    )?;
    let client = client::MoldXClient::new(config)?;
    cli.exec_with(&client).await?;
    Ok(())
}
