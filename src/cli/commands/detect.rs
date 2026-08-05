use crate::config;
use crate::probe;
use anyhow::Result;

pub async fn detect(
    path: std::path::PathBuf,
    moldx_dir_override: Option<&std::path::Path>,
    bin_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    let abs = path
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("Path does not exist: {}", path.display()))?;
    let cfg = config::MoldxConfig::resolve(&abs, moldx_dir_override, bin_dir_override)?;
    let strategies = probe::detect_strategies(&cfg.probe_path, &abs).await?;
    if strategies.is_empty() {
        println!("No strategies detected for {}", abs.display());
    } else {
        println!("Detected strategies for {}:", abs.display());
        for s in &strategies {
            println!("  - {}", s);
        }
    }

    Ok(())
}
