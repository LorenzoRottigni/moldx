use crate::{client::MoldXClient, command::Command, errors::MoldXError2};
use anyhow::Result;

/// Runs a profile command for a module path.
///
/// Accepts either `<command> <path>` or `<profile> <command> <path>`.
/// Without a profile hint, the first available profile exposing the
/// command wins. Exits with the script's exit code when it is non-zero.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw positional arguments from the external subcommand.
///
/// # Returns
///
/// Ok when the command completes with exit code zero.
///
/// # Errors
///
/// Returns [`MoldXError2::RunUsage`] or [`MoldXError2::TooManyArguments`] on
/// malformed arguments, [`MoldXError2::PathNotFound`] if the path does not
/// exist, [`MoldXError2::ProfileNotAvailable`] if the hinted profile does
/// not apply to the path, [`MoldXError2::CommandNotFoundInProfile`] or
/// [`MoldXError2::CommandNotFound`] if the command is unknown, and any error
/// raised while executing the script.
pub async fn run(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        return Err(MoldXError2::RunUsage.into());
    }
    if args.len() > 3 {
        return Err(MoldXError2::TooManyArguments.into());
    }

    let (profile_hint, command_name, path) = if args.len() == 2 {
        (None, args[0].clone(), std::path::PathBuf::from(&args[1]))
    } else {
        (
            Some(args[0].clone()),
            args[1].clone(),
            std::path::PathBuf::from(&args[2]),
        )
    };

    if !path.exists() {
        return Err(MoldXError2::PathNotFound { path, kind: "module" }.into());
    }

    let available_profiles = client.profiles_for_module(&path);

    let command: Command = if let Some(profile_name) = profile_hint {
        let profile = available_profiles
            .iter()
            .find(|candidate| candidate.name == profile_name)
            .ok_or_else(|| MoldXError2::ProfileNotAvailable { name: profile_name, path: path.clone() })?;
        profile
            .get_command(&command_name)
            .ok_or_else(|| MoldXError2::CommandNotFoundInProfile { name: command_name, profile: profile.name.clone() })?
    } else {
        available_profiles
            .iter()
            .find_map(|profile| profile.get_command(&command_name))
            .ok_or_else(|| MoldXError2::CommandNotFound { name: command_name, path: path.clone() })?
    };

    let code = client.executor.exec_blocking(&command.path, &path).await?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}
