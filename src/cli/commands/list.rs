use crate::config;
use crate::probe;
use anyhow::Result;

pub async fn list(
    path: Option<std::path::PathBuf>,
    depth: usize,
    moldx_dir_override: Option<&std::path::Path>,
    bin_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    let abs = if let Some(p) = path {
        p.canonicalize()
            .map_err(|_| anyhow::anyhow!("Path does not exist: {}", p.display()))?
    } else {
        std::env::current_dir().map_err(anyhow::Error::from)?
    };
    let cfg = config::MoldxConfig::resolve(&abs, moldx_dir_override, bin_dir_override)?;
    let modules = probe::discover_modules(&abs, &cfg, depth).await?;
    if modules.is_empty() {
        println!("No modules found under {}", abs.display());
    } else {
        for m in &modules {
            println!("{}", m.path.display());
            let mut strats: Vec<_> = m.strategies.iter().collect();
            strats.sort_by_key(|(s, _)| s.as_str());
            for (strategy, commands) in strats {
                println!("  [{}]  {}", strategy, commands.join("  "));
            }
        }
    }

    Ok(())
}
