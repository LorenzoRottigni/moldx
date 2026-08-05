use crate::config;
use crate::executor;
use crate::probe;
use anyhow::{bail, Result};
use probe::AGNOSTIC_STRATEGY;
use std::path::PathBuf;

pub async fn run(
    args: Vec<String>,
    moldx_dir_override: Option<&std::path::Path>,
    bin_dir_override: Option<&std::path::Path>,
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
        (None, args[0].clone(), PathBuf::from(&args[1]))
    } else {
        (
            Some(args[0].clone()),
            args[1].clone(),
            PathBuf::from(&args[2]),
        )
    };
    let abs = path
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("Path does not exist: {}", path.display()))?;

    let cfg = config::MoldxConfig::resolve(&abs, moldx_dir_override, bin_dir_override)?;

    validate_name(&command, "command")?;
    if let Some(ref hint) = strategy_hint {
        validate_name(hint, "strategy")?;
    }

    let detected = probe::detect_strategies(&cfg.probe_path, &abs).await?;

    let (script, _strategy_label) = if let Some(hint) = strategy_hint {
        if !detected.is_empty() && !detected.iter().any(|s| s == &hint) {
            bail!(
                "Strategy '{}' is not available for {}.\nAvailable: {}",
                hint,
                abs.display(),
                if detected.is_empty() {
                    "none".to_string()
                } else {
                    detected.join(", ")
                }
            );
        }

        let variant_script = cfg.bin_dir.join(&command).join(format!("{}.sh", hint));
        if !variant_script.exists() {
            let available = probe::available_strategies_for_command(&cfg.bin_dir, &command);
            bail!(
                "Command '{}' has no '{}' variant.\nAvailable strategies for this command: {}",
                command,
                hint,
                if available.is_empty() {
                    "none".into()
                } else {
                    available.join(", ")
                }
            );
        }
        (variant_script, hint)
    } else {
        // Try detected strategy variants first (probe order), then fall back to agnostic.
        let mut selected: Option<(PathBuf, String)> = None;
        for strategy in &detected {
            let variant_script = cfg.bin_dir.join(&command).join(format!("{}.sh", strategy));
            if variant_script.exists() {
                selected = Some((variant_script, strategy.clone()));
                break;
            }
        }

        if let Some(found) = selected {
            found
        } else {
            let agnostic_script = cfg.bin_dir.join(format!("{}.sh", command));
            if agnostic_script.exists() {
                (agnostic_script, AGNOSTIC_STRATEGY.to_string())
            } else {
                let available_variants =
                    probe::available_strategies_for_command(&cfg.bin_dir, &command);
                let available_agnostic = probe::has_agnostic_command(&cfg.bin_dir, &command);
                if !available_variants.is_empty() && detected.is_empty() {
                    bail!(
                                "Command '{}' requires a strategy variant, but no strategies were detected for {}.\nAvailable variants: {}",
                                command,
                                abs.display(),
                                available_variants.join(", ")
                            );
                }

                let mut available = Vec::new();
                if available_agnostic {
                    available.push(AGNOSTIC_STRATEGY.to_string());
                }
                available.extend(available_variants);
                bail!(
                    "Command '{}' not found for {}.\nAvailable strategy variants: {}",
                    command,
                    abs.display(),
                    if available.is_empty() {
                        "none".into()
                    } else {
                        available.join(", ")
                    }
                );
            }
        }
    };

    let code = executor::execute_blocking(&script, &abs).await?;
    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}

/// Reject names that contain path separators or are relative references.
///
/// Strategy and command names must be plain identifiers so that
/// `bin_dir.join(command).join(strategy)` cannot escape the bin directory via
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
