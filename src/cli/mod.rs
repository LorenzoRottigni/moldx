//! CLI argument definitions for moldx.
//!
//! Built with [clap](https://docs.rs/clap). Global options (`--moldx-dir`,
//! `--bin-dir`) are also readable from the `MOLDX_DIR` and `MOLDX_BIN_DIR`
//! environment variables so they can be set once in a shell profile for a
//! project-wide override.

pub mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Top-level CLI structure parsed by clap.
#[derive(Parser)]
#[command(
    name = "moldx",
    about = "Technology-agnostic orchestration engine for submodule lifecycle management",
    long_about = "Standardizes submodule lifecycle management through user-defined shell-based strategies.\n\nUsage: moldx [strategy] <command> <path>"
)]
pub struct Cli {
    /// Override the .moldx directory location (or set MOLDX_DIR env var)
    #[arg(long, env = "MOLDX_DIR", global = true)]
    pub moldx_dir: Option<String>,

    /// Override the bin directory location (or set MOLDX_BIN_DIR env var)
    #[arg(long, env = "MOLDX_BIN_DIR", global = true)]
    pub bin_dir: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// All subcommands exposed by moldx.
#[derive(Subcommand)]
pub enum Commands {
    /// Launch the interactive terminal UI
    Ui,

    /// Detect available strategies for a given path
    Detect {
        /// Target module path
        path: PathBuf,
    },

    /// List all discovered modules under a root path
    List {
        /// Root path to scan (defaults to current directory)
        path: Option<PathBuf>,

        /// Maximum directory depth to scan
        #[arg(long, default_value = "3")]
        depth: usize,
    },

    /// Create a new .moldx/ template directory in the current working directory
    Init,

    /// Run a command: moldx [strategy] <command> <path>
    /// Strategy is optional; if omitted, the best matching strategy variant is used.
    #[command(external_subcommand)]
    Run(Vec<String>),

    /// Scaffold command scripts in .moldx/bin
    New {
        /// Arguments: either `<command>` or `<strategy> <command>`
        #[arg(required = true, num_args = 1..=2)]
        args: Vec<String>,
    },
}
