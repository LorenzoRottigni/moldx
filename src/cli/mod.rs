use clap::{Subcommand};
use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

use crate::client::MoldXClient;

pub mod commands;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(
        long = "moldx-dir",
        env = "MOLDX_MOLDX_DIR",
        global = true,
        default_value = ".moldx"
    )]
    pub moldx_dir: String,

    #[arg(
        long = "strategies-dir-name",
        env = "MOLDX_STRATEGIES_DIR_NAME",
        global = true,
        default_value = "strategies"
    )]
    pub strategies_dir_name: String,

    #[arg(
        long = "bin-dir-name",
        env = "MOLDX_BIN_DIR_NAME",
        global = true,
        default_value = "bin"
    )]
    pub bin_dir_name: String,

    #[arg(
        long = "templates-dir-name",
        env = "MOLDX_TEMPLATES_DIR_NAME",
        global = true,
        default_value = "templates"
    )]
    pub templates_dir_name: String,

    #[arg(
        long = "template-dir-name",
        env = "MOLDX_TEMPLATE_DIR_NAME",
        global = true,
        default_value = "template"
    )]
    pub template_dir_name: String,
}

impl Cli {
    pub async fn exec_with(self, client: MoldXClient) -> Result<()> {
        match self.command.unwrap_or(Command::Ui) {
            // moldx [ui]
            Command::Ui => {
                commands::ui::ui(&client).await?;
            }

            // moldx detect <path>
            Command::Detect { path } => {
                commands::detect::detect(&client, path).await?;
            }

            // moldx list [<path>] [--depth <depth>]
            Command::List => {
                commands::list::list(&client).await?;
            }

            // moldx new [] [] <>
            Command::New { args } => {
                commands::new::new(&client, args).await?;
            }

            // moldx init
            Command::Init => {
                commands::init::init(&client).await?;
            }

            // moldx [strategy] <command> <path>
            Command::Run(args) => {
                commands::run::run(&client, args).await?;
            }
        }
        Ok(())
    }
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
    List,

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
