use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub dir: PathBuf,
    pub strategies: Vec<usize>,
}

impl Module {
    pub fn new(dir: PathBuf, strategies: Vec<usize>) -> Self {
        let name = dir.to_string_lossy().to_string();
        Self {
            name,
            dir,
            strategies
        }
    }
}