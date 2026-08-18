use crate::{client::MoldXClient, command::Command, errors::MoldXError};
use anyhow::Result;


pub async fn run(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        return Err(MoldXError::RunUsage.into());
    }
    if args.len() > 3 {
        return Err(MoldXError::TooManyArguments.into());
    }

    let (strategy_hint, command_name, path) = if args.len() == 2 {
        (None, args[0].clone(), std::path::PathBuf::from(&args[1]))
    } else {
        (
            Some(args[0].clone()),
            args[1].clone(),
            std::path::PathBuf::from(&args[2]),
        )
    };

    if !path.exists() {
        return Err(MoldXError::PathNotFound { path }.into());
    }

    let available_strategies = client.strategies_for_module(&path);

    let command: Command = if let Some(strategy_name) = strategy_hint {
        let strategy = available_strategies
            .iter()
            .find(|candidate| candidate.name == strategy_name)
            .ok_or_else(|| MoldXError::StrategyNotAvailable { name: strategy_name, path: path.clone() })?;
        strategy
            .get_command(&command_name)
            .ok_or_else(|| MoldXError::CommandNotFoundInStrategy { name: command_name, strategy: strategy.name.clone() })?
    } else {
        available_strategies
            .iter()
            .find_map(|strategy| strategy.get_command(&command_name))
            .ok_or_else(|| MoldXError::CommandNotFound { name: command_name, path: path.clone() })?
    };

    let code = client.executor.exec_blocking(&command.dir, &path).await?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}
