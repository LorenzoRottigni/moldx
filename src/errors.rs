use std::path::PathBuf;
use thiserror::Error;

use crate::types::Entity;

#[derive(Error, Debug)]
pub enum MoldXError {
    #[error("unable to determine current directory")]
    CurrentDir,

    #[error("unable to canonicalize path: {path}")]
    Canonicalize { path: PathBuf },

    #[error("invalid strategy directory: {path}")]
    InvalidStrategyDir { path: PathBuf },

    #[error("Invalid {entity} name: {name}")]
    InvalidName { entity: Entity, name: String },

    #[error("strategy directory has no file name: {path}")]
    StrategyDirNoFileName { path: PathBuf },

    #[error("invalid template directory: {path}")]
    InvalidTemplateDir { path: PathBuf },

    #[error("invalid strategies directory: {path}")]
    InvalidStrategiesDir { path: PathBuf },

    #[error("module directory has no file name: {path}")]
    ModuleDirNoFileName { path: PathBuf },

    #[error("unknown entity: {entity}")]
    UnknownEntity { entity: String },

    #[error("failed to spawn process: {reason}")]
    ProcessSpawnFailed { reason: String },

    #[error("failed to wait on process: {reason}")]
    ProcessWaitFailed { reason: String },

    #[error("process exited with non-zero status code: {code}")]
    ProcessNonZeroExit { code: i32 },

    #[error("terminal should remain available during TUI run")]
    TerminalUnavailable,

    #[error("path does not exist: {path}")]
    PathNotFound { path: PathBuf },

    #[error("strategy '{name}' not available for {path}")]
    StrategyNotAvailable { name: String, path: PathBuf },

    #[error("command '{name}' not found in strategy variant '{strategy}'")]
    CommandNotFoundInStrategy { name: String, strategy: String },

    #[error("command '{name}' not found for {path}")]
    CommandNotFound { name: String, path: PathBuf },

    #[error("strategy already exists: {path}")]
    StrategyAlreadyExists { path: PathBuf },

    #[error("strategy not found: {name}")]
    StrategyNotFound { name: String },

    #[error("command already exists: {path}")]
    CommandAlreadyExists { path: PathBuf },

    #[error("template not found: {name} for strategy {strategy}")]
    TemplateNotFound { name: String, strategy: String },

    #[error("strategy '{name}' does not expose a scaffoldable template")]
    NoScaffoldableTemplate { name: String },

    #[error("strategy '{name}' exposes multiple templates; pick one explicitly")]
    MultipleTemplates { name: String },

    #[error("module path already exists: {path}")]
    ModulePathAlreadyExists { path: PathBuf },

    #[error("usage: moldx new <strategy|template|module|command> ...")]
    NewUsage,

    #[error("usage: moldx [strategy] <command> <path>\n       moldx docker build ./services/auth\n       moldx build ./services/auth")]
    RunUsage,

    #[error("too many arguments; usage: moldx [strategy] <command> <path>")]
    TooManyArguments,

    #[error("usage: moldx new strategy <strategy>")]
    NewStrategyUsage,

    #[error("usage: moldx new template [strategy] <template>")]
    NewTemplateUsage,

    #[error("usage: moldx new command [strategy] <command>")]
    NewCommandUsage,

    #[error("usage: moldx new module [strategy] [template] <module-path>")]
    NewModuleUsage,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}
