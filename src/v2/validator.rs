use std::path::Path;
use anyhow::{bail, Result};

pub struct Validator {}

impl Validator {
    pub fn new() -> Self {
        Validator {}
    }

    pub fn validate_dir(&self, strategies_dir: &Path) -> Result<()> {
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
        Ok(())
    }
}