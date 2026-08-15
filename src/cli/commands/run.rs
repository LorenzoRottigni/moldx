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

    let command: Command = if let Some(strategy) = strategy_hint {
        client.get_strategy(&strategy).expect("Unable to retrieve strategy").get_command(&command_name).expect("Unable to retrieve command for given strategy")
    } else {
        client.get_default_strategies().iter().find_map(|s| s.get_command(&command_name)).expect("Unable to retrieve command from default strategy")
    };

    let code = client.executor.exec_blocking(&command.dir, &path).await?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}