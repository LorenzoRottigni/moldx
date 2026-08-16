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
        let strategy_count = self.strategies.len();
        let strategy_label = match strategy_count {
            0 => "no strategies".to_string(),
            1 => "1 strategy".to_string(),
            n => format!("{} strategies", n),
        };

        write!(f, "{} [{}] @ {}", self.name, strategy_label, self.dir.display())
    }
}
