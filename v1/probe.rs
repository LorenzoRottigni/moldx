//! Strategy discovery and module resolution.
//!
//! Strategies live under `.moldx/strategies/<strategy-name>/` and may expose:
//! - one or more shell commands under a `bin/` subdirectory
//! - one or more templates under `template/` or `templates/<name>/`
//!
//! Templates are used for two things:
//! 1. Scaffolding new modules
//! 2. Automatically matching a target path by filename
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use tokio::task;
use walkdir::WalkDir;

use crate::config::MoldxConfig;

/// Synthetic label used when a strategy is agnostic to the target path.
pub const AGNOSTIC_STRATEGY: &str = "agnostic";

/// A runnable command exposed by a strategy.
#[derive(Debug, Clone)]
pub struct CommandBinding {
    pub command: String,
    pub script_path: PathBuf,
}

/// A discovered module directory.
#[derive(Debug, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub strategies: HashMap<String, Vec<CommandBinding>>,
}

/// Concrete command resolution result.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResolvedCommand {
    pub strategy: String,
    pub command: String,
    pub script_path: PathBuf,
}

#[derive(Debug, Clone)]
struct TemplateSpec {
    name: String,
    dir: PathBuf,
    file_names: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct StrategySpec {
    name: String,
    commands: BTreeMap<String, PathBuf>,
    templates: Vec<TemplateSpec>,
}

#[derive(Debug, Clone)]
struct StrategyCatalog {
    strategies: Vec<StrategySpec>,
}

impl StrategyCatalog {
    fn load(strategies_dir: &Path) -> Result<Self> {
        if !strategies_dir.exists() {
            bail!(
                "Strategies directory not found: {}",
                strategies_dir.display()
            );
        }
        if !strategies_dir.is_dir() {
            bail!(
                "Strategies path is not a directory: {}",
                strategies_dir.display()
            );
        }

        let mut strategies = Vec::new();
        for entry in sorted_read_dir(strategies_dir)? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if is_ignored_name(&name) {
                continue;
            }

            strategies.push(StrategySpec::load(name, &path)?);
        }

        strategies.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { strategies })
    }

    fn detect_strategies_for_target(&self, target: &Path) -> Result<Vec<String>> {
        let target_files = collect_target_file_names(target)?;
        Ok(self
            .strategies
            .iter()
            .filter(|strategy| strategy.matches(&target_files))
            .map(|strategy| strategy.name.clone())
            .collect())
    }

    fn command_for_target(
        &self,
        target: &Path,
        command: &str,
        strategy_hint: Option<&str>,
    ) -> Result<ResolvedCommand> {
        let target_files = collect_target_file_names(target)?;
        let detected: Vec<&StrategySpec> = self
            .strategies
            .iter()
            .filter(|strategy| strategy.matches(&target_files))
            .collect();

        if let Some(hint) = strategy_hint {
            let strategy = self
                .strategies
                .iter()
                .find(|candidate| candidate.name == hint)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Unknown strategy '{}'. Available strategies for this command: {}",
                        hint,
                        self.available_strategies_for_command(command)
                            .into_iter()
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;

            if !strategy.is_agnostic() && !strategy.matches(&target_files) {
                bail!(
                    "Strategy '{}' is not available for {}.\nAvailable: {}",
                    hint,
                    target.display(),
                    if detected.is_empty() {
                        "none".to_string()
                    } else {
                        detected
                            .iter()
                            .map(|strategy| strategy.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
            }

            let script_path = strategy.command_script(command).ok_or_else(|| {
                anyhow::anyhow!(
                    "Command '{}' has no '{}' variant.\nAvailable strategies for this command: {}",
                    command,
                    hint,
                    self.available_strategies_for_command(command)
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

            return Ok(ResolvedCommand {
                strategy: if strategy.is_agnostic() {
                    AGNOSTIC_STRATEGY.to_string()
                } else {
                    strategy.name.clone()
                },
                command: command.to_string(),
                script_path,
            });
        }

        for strategy in &detected {
            if let Some(script_path) = strategy.command_script(command) {
                return Ok(ResolvedCommand {
                    strategy: strategy.name.clone(),
                    command: command.to_string(),
                    script_path,
                });
            }
        }

        if let Some(script_path) = self.agnostic_command_script(command) {
            return Ok(ResolvedCommand {
                strategy: AGNOSTIC_STRATEGY.to_string(),
                command: command.to_string(),
                script_path,
            });
        }

        let available_variants = self.available_strategies_for_command(command);
        if !available_variants.is_empty() && detected.is_empty() {
            bail!(
                "Command '{}' requires a matching strategy, but none were detected for {}.\nAvailable variants: {}",
                command,
                target.display(),
                available_variants.into_iter().collect::<Vec<_>>().join(", ")
            );
        }

        bail!(
            "Command '{}' not found for {}.\nAvailable strategy variants: {}",
            command,
            target.display(),
            if available_variants.is_empty() {
                "none".to_string()
            } else {
                available_variants
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
    }

    fn discovered_module(&self, path: &Path) -> Result<Option<Module>> {
        let target_files = collect_target_file_names(path)?;
        let matching: Vec<&StrategySpec> = self
            .strategies
            .iter()
            .filter(|strategy| strategy.matches(&target_files))
            .collect();

        if matching.is_empty() {
            return Ok(None);
        }

        let mut strategy_map: HashMap<String, Vec<CommandBinding>> = HashMap::new();
        for strategy in &matching {
            let commands = strategy.command_bindings();
            if !commands.is_empty() {
                strategy_map.insert(strategy.name.clone(), commands);
            }
        }

        let agnostic_commands = self.agnostic_command_bindings();
        if !agnostic_commands.is_empty() {
            strategy_map.insert(AGNOSTIC_STRATEGY.to_string(), agnostic_commands);
        }

        if strategy_map.is_empty() {
            return Ok(None);
        }

        Ok(Some(Module {
            path: path.to_path_buf(),
            strategies: strategy_map,
        }))
    }

    fn agnostic_command_script(&self, command: &str) -> Option<PathBuf> {
        self.agnostic_strategies()
            .into_iter()
            .find_map(|strategy| strategy.command_script(command))
    }

    fn agnostic_command_bindings(&self) -> Vec<CommandBinding> {
        let mut commands = BTreeMap::new();
        for strategy in self.agnostic_strategies() {
            for (command, script_path) in &strategy.commands {
                commands
                    .entry(command.clone())
                    .or_insert_with(|| CommandBinding {
                        command: command.clone(),
                        script_path: script_path.clone(),
                    });
            }
        }
        commands.into_values().collect()
    }

    fn agnostic_strategies(&self) -> impl Iterator<Item = &StrategySpec> {
        self.strategies
            .iter()
            .filter(|strategy| strategy.is_agnostic())
    }

    fn available_strategies_for_command(&self, command: &str) -> BTreeSet<String> {
        let mut strategies = BTreeSet::new();
        for strategy in &self.strategies {
            if strategy.commands.contains_key(command) {
                strategies.insert(if strategy.is_agnostic() {
                    AGNOSTIC_STRATEGY.to_string()
                } else {
                    strategy.name.clone()
                });
            }
        }
        strategies
    }
}

impl StrategySpec {
    fn load(name: String, strategy_dir: &Path) -> Result<Self> {
        let commands = collect_command_scripts(strategy_dir)?;
        let templates = collect_templates(strategy_dir)?;
        Ok(Self {
            name,
            commands,
            templates,
        })
    }

    fn is_agnostic(&self) -> bool {
        self.templates.is_empty()
            || self
                .templates
                .iter()
                .all(|template| template.file_names.is_empty())
    }

    fn matches(&self, target_files: &BTreeSet<String>) -> bool {
        self.templates
            .iter()
            .filter(|template| !template.file_names.is_empty())
            .any(|template| template.file_names.is_subset(target_files))
    }

    fn command_script(&self, command: &str) -> Option<PathBuf> {
        self.commands.get(command).cloned()
    }

    fn command_bindings(&self) -> Vec<CommandBinding> {
        self.commands
            .iter()
            .map(|(command, script_path)| CommandBinding {
                command: command.clone(),
                script_path: script_path.clone(),
            })
            .collect()
    }

    #[allow(dead_code)]
    fn template_count(&self) -> usize {
        self.templates
            .iter()
            .filter(|template| !template.file_names.is_empty())
            .count()
    }

    #[allow(dead_code)]
    fn template_names(&self) -> Vec<String> {
        self.templates
            .iter()
            .filter(|template| !template.file_names.is_empty())
            .map(|template| template.name.clone())
            .collect()
    }

    fn template_dir(&self, template_name: Option<&str>) -> Result<PathBuf> {
        let non_empty_templates: Vec<&TemplateSpec> = self
            .templates
            .iter()
            .filter(|template| !template.file_names.is_empty())
            .collect();

        match template_name {
            Some(name) => self
                .templates
                .iter()
                .find(|template| template.name == name && !template.file_names.is_empty())
                .map(|template| template.dir.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Template '{}' not found for strategy '{}'. Available templates: {}",
                        name,
                        self.name,
                        if non_empty_templates.is_empty() {
                            "none".to_string()
                        } else {
                            non_empty_templates
                                .iter()
                                .map(|template| template.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    )
                }),
            None => {
                if non_empty_templates.len() == 1 {
                    Ok(non_empty_templates[0].dir.clone())
                } else if non_empty_templates.is_empty() {
                    bail!(
                        "Strategy '{}' does not expose a scaffoldable template.",
                        self.name
                    );
                } else {
                    bail!(
                        "Strategy '{}' exposes multiple templates. Pick one explicitly: {}",
                        self.name,
                        non_empty_templates
                            .iter()
                            .map(|template| template.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
    }
}

impl TemplateSpec {
    fn load(name: String, template_dir: &Path) -> Result<Self> {
        Ok(Self {
            name,
            dir: template_dir.to_path_buf(),
            file_names: collect_target_file_names(template_dir)?,
        })
    }
}

/// Detect strategies for one target path by matching template filenames.
pub async fn detect_strategies(strategies_dir: &Path, target: &Path) -> Result<Vec<String>> {
    let catalog = StrategyCatalog::load(strategies_dir)?;
    let target = target.to_path_buf();
    task::spawn_blocking(move || catalog.detect_strategies_for_target(&target))
        .await
        .map_err(|error| anyhow::anyhow!(error))?
}

/// Return the sorted list of strategies that expose a given command.
#[allow(dead_code)]
pub fn available_strategies_for_command(
    strategies_dir: &Path,
    command: &str,
) -> Result<Vec<String>> {
    let catalog = StrategyCatalog::load(strategies_dir)?;
    Ok(catalog
        .available_strategies_for_command(command)
        .into_iter()
        .collect())
}

/// Resolve a command script for a target path.
pub fn resolve_command(
    strategies_dir: &Path,
    target: &Path,
    command: &str,
    strategy_hint: Option<&str>,
) -> Result<ResolvedCommand> {
    let catalog = StrategyCatalog::load(strategies_dir)?;
    catalog.command_for_target(target, command, strategy_hint)
}

/// Scaffold a module directory from a strategy template.
pub fn scaffold_module(
    strategies_dir: &Path,
    strategy_name: &str,
    template_name: Option<&str>,
    target: &Path,
) -> Result<()> {
    let catalog = StrategyCatalog::load(strategies_dir)?;
    let strategy = catalog
        .strategies
        .iter()
        .find(|candidate| candidate.name == strategy_name)
        .ok_or_else(|| anyhow::anyhow!("Strategy '{}' not found", strategy_name))?;

    let template_dir = strategy.template_dir(template_name)?;
    scaffold_template_dir(&template_dir, target)
}

/// Walk `root` and return every discovered module.
pub async fn discover_modules(
    root: &Path,
    config: &MoldxConfig,
    max_depth: usize,
) -> Result<Vec<Module>> {
    let catalog = StrategyCatalog::load(&config.strategies_dir)?;
    let root = root.to_path_buf();
    task::spawn_blocking(move || discover_modules_sync(root, catalog, max_depth))
        .await
        .map_err(|error| anyhow::anyhow!(error))?
}

fn discover_modules_sync(
    root: PathBuf,
    catalog: StrategyCatalog,
    max_depth: usize,
) -> Result<Vec<Module>> {
    let entries: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| is_walkable_entry(entry))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.into_path())
        .collect();

    let mut modules = Vec::new();
    for path in entries {
        if let Some(module) = catalog.discovered_module(&path)? {
            modules.push(module);
        }
    }

    modules.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(modules)
}

fn collect_command_scripts(strategy_dir: &Path) -> Result<BTreeMap<String, PathBuf>> {
    let mut scripts = BTreeMap::new();

    let bin_dir = strategy_dir.join("bin");
    if bin_dir.is_dir() {
        for entry in sorted_read_dir(&bin_dir)? {
            let path = entry.path();
            if path.is_file() && is_shell_script(&path) {
                if let Some(command) = path.file_stem().and_then(|stem| stem.to_str()) {
                    scripts.entry(command.to_string()).or_insert(path);
                }
            }
        }
    }

    Ok(scripts)
}

fn collect_templates(strategy_dir: &Path) -> Result<Vec<TemplateSpec>> {
    let mut templates = Vec::new();

    let singular = strategy_dir.join("template");
    if singular.is_dir() {
        templates.push(TemplateSpec::load("template".to_string(), &singular)?);
    }

    let plural = strategy_dir.join("templates");
    if plural.is_dir() {
        for entry in sorted_read_dir(&plural)? {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                templates.push(TemplateSpec::load(name, &path)?);
            }
        }
    }

    templates.sort_by(|a, b| a.file_names.len().cmp(&b.file_names.len()));
    Ok(templates)
}

fn collect_target_file_names(root: &Path) -> Result<BTreeSet<String>> {
    if root.is_file() {
        let mut names = BTreeSet::new();
        if let Some(name) = root.file_name().and_then(|name| name.to_str()) {
            names.insert(name.to_string());
        }
        return Ok(names);
    }

    let mut names = BTreeSet::new();
    for entry in sorted_read_dir(root)? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if name.starts_with('.') {
                continue;
            }
            names.insert(name.to_string());
        }
    }

    Ok(names)
}

fn scaffold_template_dir(template_dir: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        if !target.is_dir() {
            bail!(
                "Target path exists and is not a directory: {}",
                target.display()
            );
        }
        if std::fs::read_dir(target)?.next().is_some() {
            bail!(
                "Target path already exists and is not empty: {}",
                target.display()
            );
        }
    } else {
        std::fs::create_dir_all(target)?;
    }

    for entry in WalkDir::new(template_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let relative = match entry.path().strip_prefix(template_dir) {
            Ok(relative) => relative,
            Err(_) => continue,
        };
        let destination = target.join(relative);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(entry.path(), &destination)?;
    }

    Ok(())
}

fn sorted_read_dir(path: &Path) -> Result<Vec<std::fs::DirEntry>> {
    let mut entries = std::fs::read_dir(path)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(entries)
}

fn is_shell_script(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("sh")
}

fn is_ignored_name(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

fn is_walkable_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    let name = entry.file_name().to_str().unwrap_or("");
    !is_ignored_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_script(path: &Path, body: &str) {
        std::fs::write(
            path,
            format!("#!/usr/bin/env bash\nset -euo pipefail\n{}\n", body),
        )
        .unwrap();
    }

    fn make_catalog() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let strategies = tmp.path().join(".moldx/strategies");
        std::fs::create_dir_all(&strategies).unwrap();
        (tmp, strategies)
    }

    #[tokio::test]
    async fn detect_strategies_matches_template_filenames() {
        let (tmp, strategies) = make_catalog();
        let docker = strategies.join("docker");
        std::fs::create_dir_all(docker.join("bin")).unwrap();
        std::fs::create_dir_all(docker.join("template")).unwrap();
        std::fs::write(docker.join("template/Dockerfile"), "").unwrap();
        write_script(&docker.join("bin/build.sh"), "echo docker");

        let module = tmp.path().join("module");
        std::fs::create_dir_all(&module).unwrap();
        std::fs::write(module.join("Dockerfile"), "").unwrap();

        let result = detect_strategies(&strategies, &module).await.unwrap();
        assert_eq!(result, vec!["docker"]);
    }

    #[tokio::test]
    async fn detect_strategies_returns_empty_for_non_matching_path() {
        let (_tmp, strategies) = make_catalog();
        let docker = strategies.join("docker");
        std::fs::create_dir_all(docker.join("bin")).unwrap();
        std::fs::create_dir_all(docker.join("template")).unwrap();
        std::fs::write(docker.join("template/Dockerfile"), "").unwrap();

        let module = TempDir::new().unwrap();
        let result = detect_strategies(&strategies, module.path()).await.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn available_strategies_for_command_lists_matching_strategies() {
        let (_tmp, strategies) = make_catalog();
        let docker = strategies.join("docker");
        std::fs::create_dir_all(docker.join("bin")).unwrap();
        std::fs::create_dir_all(docker.join("template")).unwrap();
        std::fs::write(docker.join("template/Dockerfile"), "").unwrap();
        write_script(&docker.join("bin/build.sh"), "echo docker");

        let node = strategies.join("node");
        std::fs::create_dir_all(node.join("bin")).unwrap();
        std::fs::create_dir_all(node.join("template")).unwrap();
        std::fs::write(node.join("template/package.json"), "").unwrap();
        write_script(&node.join("bin/build.sh"), "echo node");

        let variants = available_strategies_for_command(&strategies, "build").unwrap();
        assert_eq!(variants, vec!["docker".to_string(), "node".to_string()]);
    }

    #[test]
    fn agnostic_commands_are_exposed_from_empty_templates() {
        let (_tmp, strategies) = make_catalog();
        let default = strategies.join("default");
        std::fs::create_dir_all(default.join("bin")).unwrap();
        std::fs::create_dir_all(default.join("template")).unwrap();
        write_script(&default.join("bin/diff.sh"), "echo diff");

        let module = TempDir::new().unwrap();
        std::fs::write(module.path().join("Dockerfile"), "").unwrap();
        let catalog = StrategyCatalog::load(&strategies).unwrap();
        let resolved = catalog
            .command_for_target(module.path(), "diff", None)
            .unwrap();
        assert_eq!(resolved.strategy, AGNOSTIC_STRATEGY);
    }

    #[tokio::test]
    async fn discover_modules_surfaces_matching_strategies_and_commands() {
        let (tmp, strategies) = make_catalog();
        let docker = strategies.join("docker");
        std::fs::create_dir_all(docker.join("bin")).unwrap();
        std::fs::create_dir_all(docker.join("template")).unwrap();
        std::fs::write(docker.join("template/Dockerfile"), "").unwrap();
        write_script(&docker.join("bin/build.sh"), "echo docker/build");

        let default = strategies.join("default");
        std::fs::create_dir_all(default.join("bin")).unwrap();
        std::fs::create_dir_all(default.join("template")).unwrap();
        write_script(&default.join("bin/diff.sh"), "echo agnostic/diff");

        let module = tmp.path().join("modules/service");
        std::fs::create_dir_all(&module).unwrap();
        std::fs::write(module.join("Dockerfile"), "").unwrap();

        let moldx = tmp.path().join(".moldx");
        let cfg = MoldxConfig::resolve(tmp.path(), Some(&moldx), Some(&strategies)).unwrap();
        let modules = discover_modules(tmp.path(), &cfg, 3).await.unwrap();
        assert_eq!(modules.len(), 1);
        let strategies_map = &modules[0].strategies;
        assert!(strategies_map.contains_key("docker"));
        assert!(strategies_map.contains_key(AGNOSTIC_STRATEGY));
    }

    #[test]
    fn resolve_command_prefers_detected_strategy() {
        let (_tmp, strategies) = make_catalog();
        let docker = strategies.join("docker");
        std::fs::create_dir_all(docker.join("bin")).unwrap();
        std::fs::create_dir_all(docker.join("template")).unwrap();
        std::fs::write(docker.join("template/Dockerfile"), "").unwrap();
        write_script(&docker.join("bin/build.sh"), "echo docker/build");

        let module = TempDir::new().unwrap();
        std::fs::write(module.path().join("Dockerfile"), "").unwrap();

        let resolved = resolve_command(&strategies, module.path(), "build", None).unwrap();
        assert_eq!(resolved.strategy, "docker");
    }

    #[test]
    fn resolve_command_errors_for_missing_path_match() {
        let (_tmp, strategies) = make_catalog();
        let docker = strategies.join("docker");
        std::fs::create_dir_all(docker.join("bin")).unwrap();
        std::fs::create_dir_all(docker.join("template")).unwrap();
        std::fs::write(docker.join("template/Dockerfile"), "").unwrap();
        write_script(&docker.join("bin/build.sh"), "echo docker/build");

        let module = TempDir::new().unwrap();
        let err = resolve_command(&strategies, module.path(), "build", None).unwrap_err();
        assert!(err.to_string().contains("requires a matching strategy"));
    }
}
