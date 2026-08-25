use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

use crate::client::MoldXClient;
use crate::constants::DEFAULT_BIN_DIR_NAME;
use crate::constants::DEFAULT_MAX_RESOLUTION_DEPTH;
use crate::constants::DEFAULT_STRATEGIES_DIR_NAME;
use crate::constants::DEFAULT_TEMPLATES_DIR_NAME;
use crate::constants::DEFAULT_TEMPLATE_DIR_NAME;
use crate::constants::MOLDX_DIR_NAME;

pub mod commands;

/// Command line interface for MoldX.
///
/// Global options configure the MoldX project layout and can also be
/// provided through environment variables.
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(
        long = "moldx-dir",
        env = "MOLDX_DIR",
        global = true,
        default_value = MOLDX_DIR_NAME
    )]
    pub moldx_dir: String,

    #[arg(
        long = "strategies-dir-name",
        env = "MOLDX_STRATEGIES_DIR_NAME",
        global = true,
        default_value = DEFAULT_STRATEGIES_DIR_NAME
    )]
    pub strategies_dir_name: String,

    #[arg(
        long = "bin-dir-name",
        env = "MOLDX_BIN_DIR_NAME",
        global = true,
        default_value = DEFAULT_BIN_DIR_NAME
    )]
    pub bin_dir_name: String,

    #[arg(
        long = "templates-dir-name",
        env = "MOLDX_TEMPLATES_DIR_NAME",
        global = true,
        default_value = DEFAULT_TEMPLATES_DIR_NAME
    )]
    pub templates_dir_name: String,

    #[arg(
        long = "template-dir-name",
        env = "MOLDX_TEMPLATE_DIR_NAME",
        global = true,
        default_value = DEFAULT_TEMPLATE_DIR_NAME
    )]
    pub template_dir_name: String,

    #[arg(long = "modules-dir", env = "MOLDX_MODULES_DIR", global = true)]
    pub modules_dir: Option<String>,

    #[arg(
        long = "max-resolution-depth",
        env = "MOLDX_MAX_RESOLUTION_DEPTH",
        global = true,
        default_value_t = DEFAULT_MAX_RESOLUTION_DEPTH
    )]
    pub max_resolution_depth: usize,
}

impl Cli {
    /// Dispatches the parsed subcommand with the given client.
    ///
    /// Defaults to launching the interactive UI when no subcommand is given.
    ///
    /// # Arguments
    ///
    /// * `client` - The initialized MoldX client.
    ///
    /// # Returns
    ///
    /// Ok when the selected subcommand completes successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the selected subcommand fails.
    pub async fn exec_with(self, client: &MoldXClient) -> Result<()> {
        self.command.unwrap_or(Command::Ui).exec_with(&client).await
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
        #[arg(required = true, num_args = 1..=3)]
        args: Vec<String>,
    },
}

impl Command {
    async fn exec_with(self, client: &MoldXClient) -> Result<()> {
        match self {
            Self::Ui => commands::ui::ui(client).await,
            Self::Detect { path } => commands::detect::detect(client, path).await,
            Self::List => commands::list::list(client).await,
            Self::Init => commands::init::init(client).await,
            Self::New { args } => commands::new::new(client, args).await,
            Self::Run(args) => commands::run::run(client, args).await,
        }
    }
}
