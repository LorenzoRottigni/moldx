use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MoldXConfig {
    pub cwd: PathBuf,
    pub moldx_dir: PathBuf,
    pub strategies_dir: PathBuf,
    pub bin_dir_name: String,
    pub template_dir_name: String,
    pub templates_dir_name: String,
}

impl MoldXConfig {
    pub fn new(
        _moldx_dir: String,
        strategies_dir_name: String,
        bin_dir_name: String,
        template_dir_name: String,
        templates_dir_name: String,
    ) -> Self {
        let moldx_dir = PathBuf::from(_moldx_dir);
        Self {
            cwd: std::env::current_dir().expect("Error: unable to determine current directory:"),
            strategies_dir: moldx_dir.join(strategies_dir_name),
            moldx_dir,
            bin_dir_name,
            template_dir_name,
            templates_dir_name,
            
        }
    }
}