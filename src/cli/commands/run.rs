use crate::config;
use crate::executor;
use crate::probe;
use anyhow::{bail, Result};

pub async fn run(
    args: Vec<String>,
    moldx_dir_override: Option<&std::path::Path>,
    strategies_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    if args.len() < 2 {
        bail!(
                    "Usage: moldx [strategy] <command> <path>\n       moldx docker build ./services/auth\n       moldx build ./services/auth"
                );
    }
    if args.len() > 3 {
        bail!("Too many arguments. Usage: moldx [strategy] <command> <path>");
    }

    // 2 args → command + path. 3 args → strategy + command + path.
    let (strategy_hint, command, path) = if args.len() == 2 {
        (None, args[0].clone(), std::path::PathBuf::from(&args[1]))
    } else {
        (
            Some(args[0].clone()),
            args[1].clone(),
            std::path::PathBuf::from(&args[2]),
        )
    };
    let abs = path
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("Path does not exist: {}", path.display()))?;

    let cfg = config::MoldxConfig::resolve(&abs, moldx_dir_override, strategies_dir_override)?;

    validate_name(&command, "command")?;
    if let Some(ref hint) = strategy_hint {
        validate_name(hint, "strategy")?;
    }

    let resolved = probe::resolve_command(
        &cfg.strategies_dir,
        &abs,
        &command,
        strategy_hint.as_deref(),
    )?;

    let code = executor::execute_blocking(&resolved.script_path, &abs).await?;
    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}

/// Reject names that contain path separators or are relative references.
///
/// Strategy and command names must be plain identifiers so that
/// `strategies_dir.join(strategy).join(command)` cannot escape the strategies directory via
/// sequences like `../../etc`.
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
