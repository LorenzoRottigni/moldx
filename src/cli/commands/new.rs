use crate::config;
use crate::probe;
use anyhow::{bail, Result};
use std::path::Path;

pub async fn new(
    args: Vec<String>,
    moldx_dir_override: Option<&std::path::Path>,
    strategies_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: moldx new module <strategy> [template] <path>");
    }

    match args[0].as_str() {
        "module" => new_module(&args[1..], moldx_dir_override, strategies_dir_override).await,
        other => bail!("Unknown new subcommand '{}'. Supported: module", other),
    }
}

async fn new_module(
    args: &[String],
    moldx_dir_override: Option<&std::path::Path>,
    strategies_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    if args.len() != 2 && args.len() != 3 {
        bail!("Usage: moldx new module <strategy> [template] <path>");
    }

    let (strategy, template, path) = if args.len() == 2 {
        (args[0].as_str(), None, Path::new(&args[1]))
    } else {
        (
            args[0].as_str(),
            Some(args[1].as_str()),
            Path::new(&args[2]),
        )
    };

    validate_name(strategy, "strategy")?;
    if let Some(template) = template {
        validate_name(template, "template")?;
    }

    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let cwd = std::env::current_dir()?;
    let cfg = config::MoldxConfig::resolve(&cwd, moldx_dir_override, strategies_dir_override)?;

    probe::scaffold_module(&cfg.strategies_dir, strategy, template, &abs)?;
    println!(
        "Scaffolded {} from {}{}",
        abs.display(),
        strategy,
        template
            .map(|name| format!("/{}", name))
            .unwrap_or_default()
    );
    Ok(())
}

fn validate_name(name: &str, kind: &str) -> Result<()> {
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        bail!(
            "Invalid {} name {:?}: must not contain path separators or be a relative reference",
            kind,
            name
        );
    }
    Ok(())
}
