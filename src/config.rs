use std::path::PathBuf;
use std::fmt::{self, Display};
use owo_colors::OwoColorize;

use crate::errors::MoldXError;

#[derive(Debug, Clone)]
pub struct MoldXConfig {
    pub cwd: PathBuf,
    pub moldx_dir: PathBuf,
    pub strategies_dir: PathBuf,
    pub bin_dir_name: String,
    pub template_dir_name: String,
    pub templates_dir_name: String,
    pub modules_dir: PathBuf,
}

impl MoldXConfig {
    pub fn new(
        _moldx_dir: String,
        strategies_dir_name: String,
        bin_dir_name: String,
        template_dir_name: String,
        templates_dir_name: String,
        modules_dir: String,
    ) -> Result<Self, MoldXError> {
        let cwd = std::env::current_dir().map_err(|_| MoldXError::CurrentDir)?;
        let moldx_dir = PathBuf::from(_moldx_dir);
        let modules_path = PathBuf::from(&modules_dir);
        let modules_dir = modules_path.canonicalize().map_err(|_| MoldXError::Canonicalize { path: modules_path })?;
        Ok(Self {
            cwd,
            strategies_dir: moldx_dir.join(strategies_dir_name),
            moldx_dir,
            bin_dir_name,
            template_dir_name,
            templates_dir_name,
            modules_dir,
        })
    }
}

impl Display for MoldXConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", "cwd:".bold().blue(), self.cwd.display())?;
        writeln!(f, "{} {}", "moldx_dir:".bold().blue(), self.moldx_dir.display())?;
        writeln!(f, "{} {}", "strategies_dir:".bold().blue(), self.strategies_dir.display())?;
        writeln!(f, "{} {}", "modules_dir:".bold().blue(), self.modules_dir.display())
    }
}
