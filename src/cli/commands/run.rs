use crate::{client::MoldXClient, command::Command};
use anyhow::{bail, Result};


pub async fn run(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        bail!(
            "Usage: moldx [strategy] <command> <path>\n       moldx docker build ./services/auth\n       moldx build ./services/auth"
        );
    }
    if args.len() > 3 {
        bail!("Too many arguments. Usage: moldx [strategy] <command> <path>");
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
        bail!("Path does not exist: {}", path.display());
    }

    let available_strategies = client.strategies_for_module(&path);

    let command: Command = if let Some(strategy_name) = strategy_hint {
        let strategy = available_strategies
            .iter()
            .find(|candidate| candidate.name == strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Strategy '{}' not available for {}", strategy_name, path.display()))?;
        strategy
            .get_command(&command_name)
            .ok_or_else(|| anyhow::anyhow!("Command '{}' not found in strategy variant '{}'", command_name, strategy.name))?
    } else {
        available_strategies
            .iter()
            .find_map(|strategy| strategy.get_command(&command_name))
            .ok_or_else(|| anyhow::anyhow!("Command '{}' not found for {}", command_name, path.display()))?
    };

    let code = client.executor.exec_blocking(&command.dir, &path).await?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}
