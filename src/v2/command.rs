use std::path::PathBuf;

use crate::v2::fs::is_shell_script;

pub struct Command {
    pub name: String,
    pub path: PathBuf,
}

impl Command {
    pub fn new(path: PathBuf) -> Option<Self> {
        if !path.is_file() || !is_shell_script(&path) {
            return None;
        }

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())?
            .to_string();

        Some(Self { name, path })
    }

}