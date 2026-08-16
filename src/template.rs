use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::fs::file_names_for_dir;

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
            "no files".to_string()
        } else {
            self.file_names
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        };

        write!(f, "{} [{}] @ {}", self.name, file_names, self.dir.display())
    }
}
