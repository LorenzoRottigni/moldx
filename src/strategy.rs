use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use anyhow::{Result};
use owo_colors::OwoColorize;

use crate::config::MoldXConfig;
use crate::fs::{sorted_read_dir};
use crate::template::{Template};
use crate::command::{Command};

#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub dir: PathBuf,
    pub commands: Vec<Command>,
    pub templates: Vec<Template>,
}

impl Strategy {
    pub fn new(strategy_dir: PathBuf, config: &MoldXConfig) -> Result<Self> {
        strategy_dir.exists() && strategy_dir.is_dir() ||
            return Err(anyhow::anyhow!("Invalid strategy directory"));
        let name = strategy_dir
            .file_name()
            .expect("Strategy directory has no file name")
            .to_string_lossy()
            .into_owned();
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
            .filter_map(|e| Command::new(e.path()))
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
