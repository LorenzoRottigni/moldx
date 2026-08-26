use std::path::PathBuf;
use thiserror::Error;

use crate::types::Entity;

#[derive(Error, Debug)]
pub enum MoldXError2 {
    #[error("[MoldX Error] Unable to read {kind} directory: {path}.")]
    PathNotFound { path: PathBuf, kind: &'static str },

    #[error("[MoldX Error] Unable to determine {entity} name from: {path}.")]
    NameResolutionFailed { path: PathBuf, entity: Entity },

    #[error("[MoldX Error] Invalid {entity} name: {name}.")]
    InvalidName { entity: Entity, name: String },

    #[error("[MoldX Error] Parent profile {parent} template must be a subset of child profile template: {child}.")]
    UnmatchedChildProfile { parent: PathBuf, child: PathBuf },

    #[error("[MoldX Error] Command must be a shell script: {path}.")]
    InvalidCommandFormat { path: PathBuf },

    #[error("[MoldX Error] Unable to retrieve CWD.")]
    CwdNotFound,

    #[error("[MoldX Error] Unable to resolve the modules root directory as parent of: {path}")]
    ModulesRootResolutionFailed { path: PathBuf },

    #[error("[MoldX Error] Unable to spawn process: {reason}")]
    ProcessSpawnFailed { reason: String },

    #[error("[MoldX Error] Path discovery for {kind} failed starting from {start}.")]
    DiscoveryFailed { start: PathBuf, kind: &'static str },

    #[error("[MoldX Error] Unknown entity: {entity}.")]
    UnknownEntity { entity: String },
    
    #[error("[MoldX Error] Terminal must remain available during TUI run")]
    TerminalUnavailable,

}

/// Errors produced by MoldX operations.
///
/// Every variant renders a human-readable message through its
/// [`std::error::Error`] implementation, covering configuration, resolution,
/// scaffolding, and process execution failures.
#[derive(Error, Debug)]
pub enum OMoldXError {
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

    #[error("template not found for strategy {strategy}")]
    TemplateNotFound { strategy: String },

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
