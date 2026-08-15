use std::path::{PathBuf};
use std::collections::{BTreeSet};
use anyhow::{Result};
use crate::v2::fs::{file_names_for_dir};

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
}