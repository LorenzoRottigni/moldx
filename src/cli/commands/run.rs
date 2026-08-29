use anyhow::Result;
use dialoguer::Select;
use std::path::PathBuf;

use crate::{cli::args::FromCommandArgs, client::MoldXClient};

pub struct RunCommandArgs {
    /// Profile hierarchy.
    pub profiles: Vec<String>,

    /// Command to execute.
    pub command: String,

    /// Module path.
    pub path: PathBuf,
}

impl FromCommandArgs for RunCommandArgs {
    fn from_command_args(mut args: Vec<String>) -> Result<Self> {
        if args.len() < 2 {
            anyhow::bail!("Expected <command> <path>");
        }

        let path = PathBuf::from(args.pop().expect("path was validated"));

        let command = args.pop().expect("command was validated");

        let profiles = args;

        Ok(Self {
            profiles,
            command,
            path,
        })
    }
}

/// Runs a profile command for a module path.
///
/// Accepts either `<command> <path>` or `<profile> <command> <path>`.
/// Without a profile hint, the first available profile exposing the
/// command wins. Exits with the script's exit code when it is non-zero.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Resolved arguments for the run command.
///
/// # Returns
///
/// Ok when the command completes with exit code zero.
///
/// # Errors
///
/// Returns [`MoldXError2::PathNotFound`] if the path does not exist,
/// [`MoldXError2::ProfileNotAvailable`] if the hinted profile does not
/// apply to the path, [`MoldXError2::CommandNotFoundInProfile`] or
/// [`MoldXError2::CommandNotFound`] if the command is unknown, and any
/// error raised while executing the script.
pub async fn run(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let args = RunCommandArgs::from_command_args(args)?;
    let commands =
        client.commands_for_module(&args.command, &args.path.to_path_buf(), &args.profiles);

    let command = if commands.len() > 1 {
        let selection = Select::new()
            .with_prompt("Multiple commands found, select one")
            .items(&commands)
            .default(0)
            .interact()?;

        &commands[selection]
    } else {
        &commands[0]
    };

    let code = client
        .executor
        .exec_blocking(&command.path, &args.path)
        .await?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}
