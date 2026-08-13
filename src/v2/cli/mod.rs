use clap::{Subcommand};
use std::path::PathBuf;
use clap::Parser;

pub fn parse_cli_command() -> Option<Command> {
    #[derive(Parser)]
    struct _Temp {
        #[command(subcommand)]
        command: Option<Command>,
    }
    
    _Temp::parse().command
}

#[derive(Subcommand)]
pub enum Command {
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
