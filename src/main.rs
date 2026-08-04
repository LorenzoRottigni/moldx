//! moldx — technology-agnostic orchestration engine.
//!
//! Entry point: parses CLI arguments and dispatches to the appropriate handler.
//! All subcommands resolve a [`config::MoldxConfig`] before doing any work so
//! that the `.moldx/` directory is guaranteed to exist.
mod cli;
mod config;
mod detector;
mod executor;
mod state;
mod ui;

use anyhow::{Result, bail};
use clap::Parser;
use cli::{Cli, Commands};
use detector::AGNOSTIC_STRATEGY;
use state::AppState;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let moldx_dir_override = cli.moldx_dir.as_deref().map(std::path::Path::new);
    let commands_dir_override = cli.commands_dir.as_deref().map(std::path::Path::new);

    match cli.command {
        Commands::Ui => {
            let cfg = config::MoldxConfig::resolve(
                &std::env::current_dir()?,
                moldx_dir_override,
                commands_dir_override,
            )?;
            let state = AppState::with_persistence(cfg.state_file_path.clone());
            ui::tui::run(cfg, state).await?;
        }

        Commands::Detect { path } => {
            let abs = canonicalize_or_err(&path)?;
            let cfg =
                config::MoldxConfig::resolve(&abs, moldx_dir_override, commands_dir_override)?;
            let strategies = detector::detect_strategies(&cfg.detector_path, &abs).await?;
            if strategies.is_empty() {
                println!("No strategies detected for {}", abs.display());
            } else {
                println!("Detected strategies for {}:", abs.display());
                for s in &strategies {
                    println!("  - {}", s);
                }
            }
        }

        Commands::List { path, depth } => {
            let root = path
                .map(Ok)
                .unwrap_or_else(|| std::env::current_dir().map_err(anyhow::Error::from))?;
            let abs = canonicalize_or_err(&root)?;
            let cfg =
                config::MoldxConfig::resolve(&abs, moldx_dir_override, commands_dir_override)?;
            let modules = detector::discover_modules(&abs, &cfg, depth).await?;
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
        }

        // moldx [strategy] <command> <path>  — strategy is optional.
        Commands::Run(args) => {
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
            let abs = canonicalize_or_err(&path)?;

            let cfg =
                config::MoldxConfig::resolve(&abs, moldx_dir_override, commands_dir_override)?;

            validate_name(&command, "command")?;
            if let Some(ref hint) = strategy_hint {
                validate_name(hint, "strategy")?;
            }

            let detected = detector::detect_strategies(&cfg.detector_path, &abs).await?;

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

                let variant_script = cfg.commands_dir.join(&command).join(format!("{}.sh", hint));
                if !variant_script.exists() {
                    let available =
                        detector::available_strategies_for_command(&cfg.commands_dir, &command);
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
                // Try detected strategy variants first (detector order), then fall back to agnostic.
                let mut selected: Option<(PathBuf, String)> = None;
                for strategy in &detected {
                    let variant_script = cfg
                        .commands_dir
                        .join(&command)
                        .join(format!("{}.sh", strategy));
                    if variant_script.exists() {
                        selected = Some((variant_script, strategy.clone()));
                        break;
                    }
                }

                if let Some(found) = selected {
                    found
                } else {
                    let agnostic_script = cfg.commands_dir.join(format!("{}.sh", command));
                    if agnostic_script.exists() {
                        (agnostic_script, AGNOSTIC_STRATEGY.to_string())
                    } else {
                        let available_variants =
                            detector::available_strategies_for_command(&cfg.commands_dir, &command);
                        let available_agnostic =
                            detector::has_agnostic_command(&cfg.commands_dir, &command);
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
        }
    }

    Ok(())
}

fn canonicalize_or_err(path: &std::path::Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|_| anyhow::anyhow!("Path does not exist: {}", path.display()))
}

/// Reject names that contain path separators or are relative references.
///
/// Strategy and command names must be plain identifiers so that
/// `commands_dir.join(command).join(strategy)` cannot escape the
/// commands directory via sequences like `../../etc`.
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
