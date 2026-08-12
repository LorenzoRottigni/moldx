use std::path::{Path, PathBuf};
use std::collections::{BTreeSet};
use anyhow::{Result};
use crate::v2::fs::{file_names_for_dir};

pub struct Template {
    pub name: String,
    pub dir: PathBuf,
    pub file_names: BTreeSet<String>,
}

impl Template {
    pub fn new(name: String, template_dir: &Path) -> Result<Self> {
        Ok(Self {
            name,
            dir: template_dir.to_path_buf(),
            file_names: file_names_for_dir(template_dir)?,
        })
    }
}