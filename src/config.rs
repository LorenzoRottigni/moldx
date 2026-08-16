use std::path::PathBuf;
use std::fmt::{self, Display};

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
    ) -> Self {
        let cwd = std::env::current_dir().expect("Error: unable to determine current directory:");
        let moldx_dir = PathBuf::from(_moldx_dir);
        Self {
            cwd,
            strategies_dir: moldx_dir.join(strategies_dir_name),
            moldx_dir,
            bin_dir_name,
            template_dir_name,
            templates_dir_name,
            modules_dir: PathBuf::from(modules_dir)
        }
    }
}

impl Display for MoldXConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MoldXConfig(cwd: {}, moldx_dir: {}, strategies_dir: {}, bin_dir_name: {}, template_dir_name: {}, templates_dir_name: {}, modules_dir: {})",
            self.cwd.display(),
            self.moldx_dir.display(),
            self.strategies_dir.display(),
            self.bin_dir_name,
            self.template_dir_name,
            self.templates_dir_name,
            self.modules_dir.display()
        )
    }
}
