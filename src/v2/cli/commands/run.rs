use crate::v2::client::MoldXClient;
use crate::v2::executor::Executor;
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

    // 2 args → command + path. 3 args → strategy + command + path.
    let (strategy_hint, command_name, path) = if args.len() == 2 {
        (None, args[0].clone(), std::path::PathBuf::from(&args[1]))
    } else {
        (
            Some(args[0].clone()),
            args[1].clone(),
            std::path::PathBuf::from(&args[2]),
        )
    };

    if let Some(strategy) = client.get_strategy(&strategy_hint.clone().unwrap_or("default".to_string())) {
        let command = strategy.get_command(command_name).expect(&format!("Command not found for strategy {:?}", strategy_hint));
        let code = Executor::exec_blocking(&command.dir, &path).await?;
        if code != 0 {
            std::process::exit(code);
        }
    }

    

    Ok(())
}