use std::path::PathBuf;
use thiserror::Error;

use crate::types::Entity;

#[derive(Error, Debug)]
pub enum MoldXError2 {
    #[error("[MoldX Error]: Unable to read {kind} dir: {path}")]
    PathNotFound { path: PathBuf, kind: &'static str },

    #[error("[MoldX Error]: Unable to determine {entity} name from: {path}")]
    NameResolutionFailed { path: PathBuf, entity: Entity },

    #[error("[MoldX Error]: Invalid {entity} name: {name}")]
    InvalidName { entity: Entity, name: String },

    #[error("[MoldX Error]: Parent profile {parent} template must be a subset of child profile template: {child}")]
    UnmatchedChildProfile { parent: PathBuf, child: PathBuf },

    #[error("[MoldX Error]: Command must be a shell script: {path}")]
    InvalidCommandFormat { path: PathBuf }
}

/// Errors produced by MoldX operations.
///
/// Every variant renders a human-readable message through its
/// [`std::error::Error`] implementation, covering configuration, resolution,
/// scaffolding, and process execution failures.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_display_all_variants() {
        let errors: Vec<String> = vec![
            MoldXError::CurrentDir.to_string(),
            MoldXError::Canonicalize {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::InvalidStrategyDir {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::InvalidName {
                entity: Entity::Strategy,
                name: "bad".into(),
            }
            .to_string(),
            MoldXError::StrategyDirNoFileName {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::InvalidTemplateDir {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::InvalidStrategiesDir {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::ModuleDirNoFileName {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::UnknownEntity { entity: "x".into() }.to_string(),
            MoldXError::ProcessSpawnFailed { reason: "r".into() }.to_string(),
            MoldXError::ProcessWaitFailed { reason: "r".into() }.to_string(),
            MoldXError::ProcessNonZeroExit { code: 1 }.to_string(),
            MoldXError::TerminalUnavailable.to_string(),
            MoldXError::PathNotFound {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::StrategyNotAvailable {
                name: "s".into(),
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::CommandNotFoundInStrategy {
                name: "c".into(),
                strategy: "s".into(),
            }
            .to_string(),
            MoldXError::CommandNotFound {
                name: "c".into(),
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::StrategyAlreadyExists {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::StrategyNotFound { name: "s".into() }.to_string(),
            MoldXError::CommandAlreadyExists {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::TemplateNotFound {
                name: "t".into(),
                strategy: "s".into(),
            }
            .to_string(),
            MoldXError::NoScaffoldableTemplate { name: "s".into() }.to_string(),
            MoldXError::MultipleTemplates { name: "s".into() }.to_string(),
            MoldXError::ModulePathAlreadyExists {
                path: PathBuf::from("/p"),
            }
            .to_string(),
            MoldXError::NewUsage.to_string(),
            MoldXError::RunUsage.to_string(),
            MoldXError::TooManyArguments.to_string(),
            MoldXError::NewStrategyUsage.to_string(),
            MoldXError::NewTemplateUsage.to_string(),
            MoldXError::NewCommandUsage.to_string(),
            MoldXError::NewModuleUsage.to_string(),
        ];
        for msg in errors {
            assert!(!msg.is_empty());
        }
    }
}
