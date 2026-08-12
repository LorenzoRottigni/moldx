use std::path::{Path, PathBuf};
use std::collections::{BTreeMap};
use anyhow::{Result};

use crate::v2::fs::{sorted_read_dir};
use crate::v2::template::{Template};
use crate::v2::command::{Command};

pub struct Strategy {
    pub name: String,
    pub commands: BTreeMap<String, PathBuf>,
    pub templates: Vec<Template>,
}

impl Strategy {
    pub fn new(name: String, strategy_dir: &Path) -> Result<Self> {
        let commands = Self::load_commands(strategy_dir)?;
        let templates = Self::load_templates(strategy_dir)?;
        Ok(Self {
            name,
            commands,
            templates,
        })
    }

    pub fn load_commands(strategy_dir: &Path) -> Result<BTreeMap<String, PathBuf>> {
        let mut scripts = BTreeMap::new();

        let bin_dir = strategy_dir.join("bin");
        if bin_dir.is_dir() {
            for entry in sorted_read_dir(&bin_dir)? {
                let path = entry.path();
                if let Some(command) = Command::new(path) {
                    scripts.insert(command.name, command.path);
                }
            }
        }

        Ok(scripts)
    }

    pub fn load_templates(strategy_dir: &Path) -> Result<Vec<Template>> {
        let mut templates = Vec::new();

        let singular = strategy_dir.join("template");
        if singular.is_dir() {
            templates.push(Template::new("template".to_string(), &singular)?);
        }

        let plural = strategy_dir.join("templates");
        if plural.is_dir() {
            for entry in sorted_read_dir(&plural)? {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    templates.push(Template::new(name, &path)?);
                }
            }
        }

        templates.sort_by(|a, b| a.file_names.len().cmp(&b.file_names.len()));
        Ok(templates)
    }
}

