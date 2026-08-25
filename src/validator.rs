use std::path::PathBuf;

use crate::{errors::MoldXError, types::Entity};

pub struct Validator {
    pub errors: Vec<MoldXError>,
}

impl Validator {
    pub fn is_dir(path: &PathBuf, entity: Entity) -> anyhow::Result<()> {
        if !path.is_dir() {
            return Err(MoldXError::PathNotFound {
                path: commands_dir,
                entity,
            }
            .into());
        }
        Ok(())
    }
}
