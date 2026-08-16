use std::path::PathBuf;
use std::fmt::{self, Display};

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

impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strategies = if self.strategies.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                self.strategies
                    .iter()
                    .map(|strategy| strategy.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        write!(
            f,
            "Module(name: {}, dir: {}, strategies: {})",
            self.name,
            self.dir.display(),
            strategies
        )
    }
}
