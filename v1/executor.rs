//! Script execution helpers for foreground CLI commands.
//!
//! [`execute_blocking`] runs a strategy script in the foreground with
//! inherited stdio, waits for it to finish, and returns the exit code.  It is
//! used by the CLI `run` subcommand.
use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// Run `script` with `module_path` as its first argument.
///
/// stdio is inherited from the parent process so the user sees output in
/// real-time.  Returns the script’s exit code; callers are responsible for
/// deciding whether a non-zero code is fatal.
pub async fn execute_blocking(script: &Path, module_path: &Path) -> Result<i32> {
    let status = Command::new("bash")
        .arg(script)
        .arg(module_path)
        .status()
        .await?;
    Ok(status.code().unwrap_or(1))
}
