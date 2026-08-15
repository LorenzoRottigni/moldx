
use crate::client::MoldXClient;
use anyhow::Result;

pub async fn list(client: &MoldXClient) -> Result<()> {
    if client.modules.is_empty() {
        println!("No modules found recursively starting from {}", client.config.cwd.to_string_lossy());
    } else {
        for m in &client.modules {
            println!("{}", m.dir.display());
            for strategy in client.strategies_for_module(&m.dir) {
                let command_names = strategy.commands
                    .iter()
                    .map(|command| command.name.as_str())
                    .collect::<Vec<_>>()
                    .join("  ");
                println!("  [{}]  {}", strategy, command_names);
            }
        }
    }

    Ok(())
}
