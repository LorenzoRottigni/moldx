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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_template_new_valid() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        let t = Template::new("docker".into(), tpl_dir.clone()).unwrap();
        assert_eq!(t.name, "docker");
        assert_eq!(t.dir, tpl_dir);
        assert!(t.file_names.contains("Dockerfile"));
    }

    #[test]
    fn test_template_new_invalid_dir() {
        let result = Template::new("x".into(), PathBuf::from("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_template_new_file_not_dir() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("not_a_dir");
        fs::write(&file, "").unwrap();
        let result = Template::new("x".into(), file);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_new_invalid_name() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        let result = Template::new("../escape".into(), tpl_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_matches_subset() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        let t = Template::new("docker".into(), tpl_dir).unwrap();

        let target = dir.path().join("module");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("Dockerfile"), "").unwrap();
        fs::write(target.join("main.rs"), "").unwrap();
        assert!(t.matches(&target));
    }

    #[test]
    fn test_template_matches_not_subset() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        fs::write(tpl_dir.join("docker-compose.yml"), "").unwrap();
        let t = Template::new("docker".into(), tpl_dir).unwrap();

        let target = dir.path().join("module");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("Dockerfile"), "").unwrap();
        assert!(!t.matches(&target));
    }

    #[test]
    fn test_template_matches_empty_file_names() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        let t = Template::new("empty".into(), tpl_dir).unwrap();
        assert!(t.file_names.is_empty());

        let target = dir.path().join("module");
        fs::create_dir(&target).unwrap();
        assert!(!t.matches(&target));
    }

    #[test]
    fn test_template_matches_nonexistent_target() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Cargo.toml"), "").unwrap();
        let t = Template::new("rust".into(), tpl_dir).unwrap();
        assert!(!t.matches(Path::new("/nonexistent_path")));
    }

    #[test]
    fn test_template_display_with_files() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        let t = Template::new("docker".into(), tpl_dir).unwrap();
        let display = t.to_string();
        assert!(display.contains("docker"));
        assert!(display.contains("Dockerfile"));
    }

    #[test]
    fn test_template_display_empty() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        let t = Template::new("empty".into(), tpl_dir).unwrap();
        let display = t.to_string();
        assert!(display.contains("no files"));
    }
}
