use crate::config;
use crate::probe;
use crate::v2::client::MoldXClient;
use anyhow::Result;

pub async fn list(
    client: MoldXClient,
    path: Option<std::path::PathBuf>,
    depth: usize
) -> Result<()> {
    let abs = if let Some(p) = path {
        p.canonicalize()
            .map_err(|_| anyhow::anyhow!("Path does not exist: {}", p.display()))?
    } else {
        std::env::current_dir().map_err(anyhow::Error::from)?
    };
    let modules = probe::discover_modules(&abs, &cfg, depth).await?;
    if modules.is_empty() {
        println!("No modules found under {}", abs.display());
    } else {
        for m in &modules {
            println!("{}", m.path.display());
            let mut strats: Vec<_> = m.strategies.iter().collect();
            strats.sort_by_key(|(s, _)| s.as_str());
            for (strategy, commands) in strats {
                let command_names = commands
                    .iter()
                    .map(|command| command.command.as_str())
                    .collect::<Vec<_>>()
                    .join("  ");
                println!("  [{}]  {}", strategy, command_names);
            }
        }
    }

    Ok(())
}
