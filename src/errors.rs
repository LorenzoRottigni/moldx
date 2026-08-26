use std::path::PathBuf;
use thiserror::Error;

use crate::types::Entity;

/// Errors produced by MoldX operations.
///
/// Every variant renders a human-readable message through its
/// [`std::error::Error`] implementation, covering configuration, resolution,
/// scaffolding, and process execution failures.
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

    #[error("[MoldX Error] Unable to determine current directory")]
    CurrentDir,

    #[error("[MoldX Error] Unable to canonicalize path: {path}")]
    Canonicalize { path: PathBuf },

    #[error("[MoldX Error] Invalid profile directory: {path}")]
    InvalidProfileDir { path: PathBuf },

    #[error("[MoldX Error] Profile directory has no file name: {path}")]
    ProfileDirNoFileName { path: PathBuf },

    #[error("[MoldX Error] Profile '{name}' not available for {path}")]
    ProfileNotAvailable { name: String, path: PathBuf },

    #[error("[MoldX Error] Command '{name}' not found in profile '{profile}'")]
    CommandNotFoundInProfile { name: String, profile: String },

    #[error("[MoldX Error] Command '{name}' not found for {path}")]
    CommandNotFound { name: String, path: PathBuf },

    #[error("[MoldX Error] Profile already exists: {path}")]
    ProfileAlreadyExists { path: PathBuf },

    #[error("[MoldX Error] Profile not found: {name}")]
    ProfileNotFound { name: String },

    #[error("[MoldX Error] Command already exists: {path}")]
    CommandAlreadyExists { path: PathBuf },

    #[error("[MoldX Error] Template not found for profile {profile}")]
    TemplateNotFound { profile: String },

    #[error("[MoldX Error] Profile '{name}' does not expose a scaffoldable template")]
    NoScaffoldableTemplate { name: String },

    #[error("[MoldX Error] Profile '{name}' exposes multiple templates; pick one explicitly")]
    MultipleTemplates { name: String },

    #[error("[MoldX Error] Module path already exists: {path}")]
    ModulePathAlreadyExists { path: PathBuf },

    #[error("[MoldX Error] Usage: moldx new <profile|module|command> ...")]
    NewUsage,

    #[error("[MoldX Error] Usage: moldx [profile] <command> <path>\n       moldx docker build ./services/auth\n       moldx build ./services/auth")]
    RunUsage,

    #[error("[MoldX Error] Too many arguments; usage: moldx [profile] <command> <path>")]
    TooManyArguments,

    #[error("[MoldX Error] Usage: moldx new profile <profile>")]
    NewProfileUsage,

    #[error("[MoldX Error] Usage: moldx new command [profile] <command>")]
    NewCommandUsage,

    #[error("[MoldX Error] Usage: moldx new module [profile] [template] <module-path>")]
    NewModuleUsage,

    #[error("[MoldX Error] Failed to wait on process: {reason}")]
    ProcessWaitFailed { reason: String },

    #[error("[MoldX Error] Process exited with non-zero status code: {code}")]
    ProcessNonZeroExit { code: i32 },

    #[error("[MoldX Error] io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("[MoldX Error] walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}
