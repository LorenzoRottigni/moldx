use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use anyhow::{Result};
use owo_colors::OwoColorize;

use crate::config::MoldXConfig;
use crate::errors::MoldXError;
use crate::fs::{sorted_read_dir, validate_name};
use crate::template::{Template};
use crate::command::{Command};
use crate::types::Entity;

#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub dir: PathBuf,
    pub commands: Vec<Command>,
    pub templates: Vec<Template>,
}

impl Strategy {
    pub fn new(strategy_dir: PathBuf, config: &MoldXConfig) -> Result<Self> {
        if !strategy_dir.exists() || !strategy_dir.is_dir() {
            return Err(MoldXError::InvalidStrategyDir { path: strategy_dir }.into());
        }
        let name = strategy_dir
            .file_name()
            .ok_or_else(|| MoldXError::StrategyDirNoFileName { path: strategy_dir.clone() })?
            .to_string_lossy()
            .into_owned();

        validate_name(name.clone(), Entity::Strategy)?;

        let commands = Self::resolve_commands(&strategy_dir, &config.bin_dir_name)?;
        let templates = Self::resolve_templates(
            &strategy_dir,
            &config.template_dir_name,
            &config.templates_dir_name,
        )?;
        Ok(Self {
            dir: strategy_dir,
            name,
            commands,
            templates,
        })
    }

    pub fn resolve_commands(strategy_dir: &Path, bin_dir_name: &str) -> Result<Vec<Command>> {
        let commands_dir = strategy_dir.join(bin_dir_name);
        if !commands_dir.is_dir() {
            return Ok(vec![]);
        }

        Ok(sorted_read_dir(&commands_dir)?
            .into_iter()
            .filter(|e| e.path().is_file())
            .filter_map(|e| Some(Command::new(e.path()).ok())?)
            .collect())
    }

    pub fn resolve_templates(
        strategy_dir: &Path,
        template_dir_name: &str,
        templates_dir_name: &str,
    ) -> Result<Vec<Template>> {
        let mut templates = Vec::new();

        let singular = strategy_dir.join(template_dir_name);
        if singular.is_dir() {
            templates.push(Template::new(template_dir_name.to_string(), singular)?);
        }

        let plural = strategy_dir.join(templates_dir_name);
        if plural.is_dir() {
            for entry in sorted_read_dir(&plural)? {
                let path = entry.path();
                if path.is_dir() {
                    templates.push(Template::new(
                        entry.file_name().to_string_lossy().into_owned(),
                        path,
                    )?);
                }
            }
        }

        Ok(templates)
    }

    pub fn is_agnostic(&self) -> bool {
        self.templates.is_empty() || self.templates.iter().all(|template| template.file_names.is_empty())
    }

    pub fn get_command(&self, name: &String) -> Option<Command> {
        self.commands.iter().find(|c| c.name == *name).cloned()
    }
}

impl Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command_names = if self.commands.is_empty() {
            "none".italic().dimmed().to_string()
        } else {
            self.commands
                .iter()
                .map(|command| command.name.cyan().to_string())
                .collect::<Vec<_>>()
                .join(&" · ".dimmed().to_string())
        };

        let template_names = if self.templates.is_empty() {
            "none".italic().dimmed().to_string()
        } else {
            self.templates
                .iter()
                .map(|template| {
                    if template.file_names.is_empty() {
                        template.name.magenta().to_string()
                    } else {
                        format!("{}{}", template.name.magenta(), format!("({})", template.file_names.len()).dimmed())
                    }
                })
                .collect::<Vec<_>>()
                .join(&" · ".dimmed().to_string())
        };

        writeln!(
            f,
            "{}{} {}",
            self.name.bold().green(),
            if self.is_agnostic() { format!(" {}", "[agnostic]".yellow()) } else { String::new() },
            format!("@ {}", self.dir.display()).dimmed()
        )?;
        writeln!(f, "  {}: {}", "commands".dimmed(), command_names)?;
        write!(f, "  {}: {}", "templates".dimmed(), template_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn make_config(strategies_dir: PathBuf) -> MoldXConfig {
        let modules_dir = strategies_dir.parent().unwrap().to_path_buf();
        MoldXConfig {
            moldx_dir: strategies_dir.parent().unwrap().join(".moldx"),
            strategies_dir,
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir,
            max_resolution_depth: 20,
        }
    }

    #[test]
    fn test_strategy_new_valid() {
        let dir = tempdir().unwrap();
        let strategies_dir = dir.path().join(".moldx").join("strategies");
        let strategy_dir = strategies_dir.join("docker");
        fs::create_dir_all(strategy_dir.join("bin")).unwrap();
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("bin").join("build.sh"), "#!/bin/bash").unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        let config = make_config(strategies_dir);
        let s = Strategy::new(strategy_dir, &config).unwrap();
        assert_eq!(s.name, "docker");
        assert_eq!(s.commands.len(), 1);
        assert_eq!(s.commands[0].name, "build");
        assert_eq!(s.templates.len(), 1);
    }

    #[test]
    fn test_strategy_new_invalid_dir() {
        let config = make_config(PathBuf::from("/nonexistent"));
        let result = Strategy::new(PathBuf::from("/nonexistent"), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_new_not_a_dir() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("strategy_file");
        fs::write(&file, "").unwrap();
        let config = make_config(dir.path().to_path_buf());
        let result = Strategy::new(file, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_commands_no_bin_dir() {
        let dir = tempdir().unwrap();
        let result = Strategy::resolve_commands(dir.path(), "bin").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_commands_with_scripts() {
        let dir = tempdir().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir(&bin_dir).unwrap();
        fs::write(bin_dir.join("build.sh"), "#!/bin/bash").unwrap();
        fs::write(bin_dir.join("test.sh"), "#!/bin/bash").unwrap();
        fs::write(bin_dir.join("not_a_script.txt"), "").unwrap();
        let commands = Strategy::resolve_commands(dir.path(), "bin").unwrap();
        assert_eq!(commands.len(), 2);
        let mut names: Vec<String> = commands.iter().map(|c| c.name.clone()).collect();
        names.sort();
        assert_eq!(names, vec!["build", "test"]);
    }

    #[test]
    fn test_resolve_templates_singular() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        let templates = Strategy::resolve_templates(dir.path(), "template", "templates").unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "template");
    }

    #[test]
    fn test_resolve_templates_plural() {
        let dir = tempdir().unwrap();
        let templates_dir = dir.path().join("templates");
        let sub1 = templates_dir.join("docker");
        let sub2 = templates_dir.join("rust");
        fs::create_dir_all(&sub1).unwrap();
        fs::create_dir_all(&sub2).unwrap();
        fs::write(sub1.join("Dockerfile"), "").unwrap();
        fs::write(sub2.join("Cargo.toml"), "").unwrap();
        let templates = Strategy::resolve_templates(dir.path(), "template", "templates").unwrap();
        assert_eq!(templates.len(), 2);
    }

    #[test]
    fn test_resolve_templates_none() {
        let dir = tempdir().unwrap();
        let templates = Strategy::resolve_templates(dir.path(), "template", "templates").unwrap();
        assert!(templates.is_empty());
    }

    #[test]
    fn test_is_agnostic_no_templates() {
        let dir = tempdir().unwrap();
        let config = make_config(dir.path().to_path_buf());
        let strategy_dir = dir.path().join("agnostic");
        fs::create_dir(&strategy_dir).unwrap();
        let s = Strategy::new(strategy_dir, &config).unwrap();
        assert!(s.is_agnostic());
    }

    #[test]
    fn test_is_agnostic_empty_template_files() {
        let dir = tempdir().unwrap();
        let config = make_config(dir.path().to_path_buf());
        let strategy_dir = dir.path().join("agnostic");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join(".gitkeep"), "").unwrap();
        let s = Strategy::new(strategy_dir, &config).unwrap();
        assert!(s.is_agnostic());
    }

    #[test]
    fn test_is_not_agnostic() {
        let dir = tempdir().unwrap();
        let config = make_config(dir.path().to_path_buf());
        let strategy_dir = dir.path().join("docker");
        fs::create_dir_all(strategy_dir.join("template")).unwrap();
        fs::write(strategy_dir.join("template").join("Dockerfile"), "").unwrap();
        let s = Strategy::new(strategy_dir, &config).unwrap();
        assert!(!s.is_agnostic());
    }

    #[test]
    fn test_get_command_found() {
        let dir = tempdir().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir(&bin_dir).unwrap();
        fs::write(bin_dir.join("build.sh"), "#!/bin/bash").unwrap();
        let config = make_config(dir.path().to_path_buf());
        let s = Strategy::new(dir.path().to_path_buf(), &config).unwrap();
        assert!(s.get_command(&"build".into()).is_some());
    }

    #[test]
    fn test_get_command_not_found() {
        let dir = tempdir().unwrap();
        let config = make_config(dir.path().to_path_buf());
        let s = Strategy::new(dir.path().to_path_buf(), &config).unwrap();
        assert!(s.get_command(&"nope".into()).is_none());
    }

    #[test]
    fn test_strategy_display() {
        let dir = tempdir().unwrap();
        let config = make_config(dir.path().to_path_buf());
        let strategy_dir = dir.path().join("mystrat");
        fs::create_dir(&strategy_dir).unwrap();
        let s = Strategy::new(strategy_dir, &config).unwrap();
        let display = s.to_string();
        assert!(display.contains("mystrat"));
        assert!(display.contains("agnos"));
    }
}
