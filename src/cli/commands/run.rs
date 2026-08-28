use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::{
    cli::args::ArgsResolver,
    client::MoldXClient,
    errors::MoldXError2,
};

#[derive(Parser)]
pub struct RunCommandArgs {
    /// Optional profile name.
    pub profile: Option<String>,

    /// Command to execute.
    pub command: String,

    /// Module path.
    pub path: Option<PathBuf>,
}

impl RunCommandArgs {
    pub async fn parse_with(
        args: Vec<String>,
        client: &MoldXClient,
    ) -> Result<Self> {
        let mut args = Self::try_parse_from(
            std::iter::once("run".to_owned()).chain(args),
        )?;

        // last arg is always the module path
        // penultimate arg is always the command
        // third-to-last arg is optional and is the profile
        // all subsequents represent profiles hierachy (profile.profiles)
        // e.g.:
        // - .moldx/profiles/python/profiles/uv/profiles/fastapi/bin/build.sh
        // - .moldx/profiles/python/profiles/pip/profiles/fastapi/bin/build.sh
        // - .moldx/profiles/docker/bin/build.sh

        // Case 1
        // command moldx build ./modules/fast-api-target
        // and ./modules/fast-api-target matches both templates of uv and pip
        // then we just ask stdin to pick one between uv and pip which are the children of python profile matching target module template

        // Case 2
        // command moldx build ./modules/fast-api-target
        // and ./modules/fast-api-target matches templates uv, pip, docker
        // then we must ask stdin the whole profiles hierachy

        // Case 3
        // we add .moldx/bin/build.sh
        // The root profile has likely no template so it will match any target module
        // So once again we must ask in the whole profiles hierachy

        // Note that also templates are hierical outer profiles templates must fit inner profiles templates recursively

        if args.path.is_none() {
            let path = ArgsResolver::new(client)
                .required(None, "Module path")
                .await?;

            args.path = Some(PathBuf::from(path));
        }

        Ok(args)
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
pub async fn run(
    client: &MoldXClient,
    args: RunCommandArgs,
) -> Result<()> {
    let commands = client.commands_for_module(&args.command, &args.path.unwrap().to_path_buf(), profile_names)












    let path = args
        .path
        .expect("RunCommandArgs must be resolved before execution");

    if !path.exists() {
        return Err(
            MoldXError2::PathNotFound {
                path,
                kind: "module",
            }
            .into(),
        );
    }

    let available_profiles = client.profiles_for_module(&path);

    let command = if let Some(profile_name) = args.profile {
        let profile = available_profiles
            .iter()
            .find(|candidate| candidate.name == profile_name)
            .ok_or_else(|| MoldXError2::ProfileNotAvailable {
                name: profile_name,
                path: path.clone(),
            })?;

        profile
            .get_command(&args.command)
            .ok_or_else(|| {
                MoldXError2::CommandNotFoundInProfile {
                    name: args.command,
                    profile: profile.name.clone(),
                }
            })?
    } else {
        available_profiles
            .iter()
            .find_map(|profile| profile.get_command(&args.command))
            .ok_or_else(|| MoldXError2::CommandNotFound {
                name: args.command,
                path: path.clone(),
            })?
    };

    let code = client
        .executor
        .exec_blocking(&command.path, &path)
        .await?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}