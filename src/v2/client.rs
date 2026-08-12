use std::path::Path;
use anyhow::Result;

use crate::v2::strategy::{Strategy};
use crate::v2::validator::{Validator};
use crate::v2::fs::{sorted_read_dir, is_ignored_name};

pub struct MoldXClient {
    strategies: Vec<Strategy>,
    validator: Validator,
}

impl MoldXClient {
    pub fn new(strategies_dir: &Path) -> Result<Self> {
        let validator = Validator::new();
        validator.validate_dir(strategies_dir).expect("Invalid strategies directory");

        let mut strategies = Vec::new();
        for entry in sorted_read_dir(strategies_dir)? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if is_ignored_name(&name) {
                continue;
            }

            strategies.push(Strategy::new(name, &path)?);
        }

        Ok(MoldXClient {
            strategies: Vec::new(),
            validator,
        })
    }
}