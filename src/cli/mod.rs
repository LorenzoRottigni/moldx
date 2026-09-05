use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

use crate::client::MoldXClient;
use crate::constants::DEFAULT_BIN_DIR_NAME;
use crate::constants::DEFAULT_MAX_RESOLUTION_DEPTH;
use crate::constants::DEFAULT_PROFILES_DIR_NAME;
use crate::constants::DEFAULT_TEMPLATE_DIR_NAME;
use crate::constants::DEFAULT_TEMPLATES_DIR_NAME;
use crate::constants::MOLDX_DIR_NAME;

/// Subcommand handler modules.
pub mod commands;

/// Command line interface for MoldX.
///
/// Global options configure the MoldX project layout and can also be
/// provided through environment variables.
#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(
        long = "skip-conflicts",
        global = true,
        help = "Automatically select the first matching command when multiple profiles expose the same command"
    )]
    pub skip_conflicts: bool,

    #[arg(
        long = "moldx-dir",
        env = "MOLDX_DIR",
        global = true,
        default_value = MOLDX_DIR_NAME
    )]
    pub moldx_dir: String,

    #[arg(
        long = "profiles-dir-name",
        env = "MOLDX_PROFILES_DIR_NAME",
        global = true,
        default_value = DEFAULT_PROFILES_DIR_NAME
    )]
    pub profiles_dir_name: String,

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
        let skip_conflicts = self.skip_conflicts;
        let command = self.command.unwrap_or(Command::Ui);
        command.exec_with(client, skip_conflicts).await
    }
}

/// Available subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Print the MoldX version.
    Version,

    /// Launch the interactive terminal UI
    Ui,

    /// Detect available profiles for a given path
    Detect {
        /// Target module path
        path: PathBuf,
    },

    /// List resolved profiles, commands, templates, and modules.
    List,

    /// Create or initialize a MoldX project structure.
    /// Supported forms: `moldx init`, `moldx init profile ...`,
    /// `moldx init command ...`, and `moldx init template ...`.
    Init {
        #[arg(required = false, num_args = 0..)]
        args: Vec<String>,
    },

    /// Run a command: moldx [profile...] <command> <path> [-- <command options...>]
    #[command(external_subcommand)]
    Run(Vec<String>),
}

/// Dispatch logic for each variant.
impl Command {
    async fn exec_with(self, client: &MoldXClient, skip_conflicts: bool) -> Result<()> {
        match self {
            Self::Version => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
            Self::Ui => commands::ui::ui(client).await,
            Self::Detect { path } => commands::detect::detect(client, path).await,
            Self::List => commands::list::list(client).await,
            Self::Init { args } => commands::init::init(client, args).await,
            Self::Run(args) => commands::run::run(client, args, skip_conflicts).await,
        }
    }
}
