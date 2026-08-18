use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use owo_colors::OwoColorize;

use anyhow::Result;

use crate::errors::MoldXError;
use crate::fs::{file_names_for_dir, validate_name};
use crate::types::Entity;

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub dir: PathBuf,
    pub file_names: BTreeSet<String>,
}

impl Template {
    pub fn new(name: String, template_dir: PathBuf) -> Result<Self> {
        if !template_dir.exists() || !template_dir.is_dir() {
            return Err(MoldXError::InvalidTemplateDir { path: template_dir }.into());
        }
        validate_name(name.clone(), Entity::Template)?;
        Ok(Self {
            name,
            file_names: file_names_for_dir(&template_dir)?,
            dir: template_dir,
        })
    }

    pub fn matches(&self, target: &Path) -> bool {
        if self.file_names.is_empty() {
            return false;
        }

        let Ok(target_files) = file_names_for_dir(target) else {
            return false;
        };

        self.file_names.is_subset(&target_files)
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_names = if self.file_names.is_empty() {
            "no files".italic().dimmed().to_string()
        } else {
            self.file_names
                .iter()
                .map(|n| n.cyan().to_string())
                .collect::<Vec<_>>()
                .join(&", ".dimmed().to_string())
        };

        write!(
            f,
            "{} [{}] {}",
            self.name.bold().magenta(),
            file_names,
            format!("@ {}", self.dir.display()).dimmed()
        )
    }
}
