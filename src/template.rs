use std::path::{PathBuf};
use std::collections::{BTreeSet};
use anyhow::{Result};
use crate::fs::{file_names_for_dir};

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub dir: PathBuf,
    pub file_names: BTreeSet<String>,
}

impl Template {
    pub fn new(name: String, template_dir: PathBuf) -> Result<Self> {
        template_dir.exists() && template_dir.is_dir() ||
            return Err(anyhow::anyhow!("Invalid template directory"));
        Ok(Self {
            name,
            file_names: file_names_for_dir(&template_dir)?,
            dir: template_dir,
        })
    }

    pub fn matches(&self, target: &PathBuf) -> bool {
        let Ok(target_files) = file_names_for_dir(target) else {
            return false;
        };

        self.file_names.is_subset(&target_files)
    }
}