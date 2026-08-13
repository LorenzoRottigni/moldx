use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MoldXConfig {
    pub strategies_dir: PathBuf,
}

impl MoldXConfig {
    pub fn new(strategies_dir: Option<String>) -> Self {
        Self {
            strategies_dir:
                if let Some(dir) = strategies_dir { PathBuf::from(dir) }
                else { PathBuf::from("./.moldx/strategies") }
        }
    }
}