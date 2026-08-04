//! Script execution helpers.
//!
//! Two entry points:
//!
//! * [`execute_blocking`] — run a strategy script in the foreground with
//!   inherited stdio, wait for it to finish, and return the exit code.  Used
//!   by the CLI `run` subcommand.
//! * [`run_and_track`] — spawn a strategy script in the background, capture
//!   its stdout/stderr into [`AppState`], and update the process status when
//!   it finishes.  Used by the TUI.
use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::state::{AppState, ProcessStatus};

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

/// Spawn `script` in the background, stream its output into `state`, and
/// update the process status when it exits.
///
/// The child process is placed in its own process group (Unix only) so that
/// a `kill_process` call terminates the entire tree of child processes, not
/// just the top-level bash interpreter.
///
/// This function is `async` and is meant to be run inside a `tokio::spawn`
/// call; it returns only after the child exits.
pub async fn run_and_track(
    id: u64,
    script: std::path::PathBuf,
    module_path: std::path::PathBuf,
    state: AppState,
) {
    let mut cmd = Command::new("bash");
    cmd.arg(&script)
        .arg(&module_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Own process group so killing the group also terminates child processes
    #[cfg(unix)]
    {
        cmd.process_group(0);
    }
    let child = cmd.spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            state.update_status(id, ProcessStatus::Failed(e.to_string()));
            return;
        }
    };

    let pid = child.id();
    state.update_pid(id, pid);

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let state2 = state.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                state2.append_output(id, line);
            }
        });
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let state3 = state.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                state3.append_output(id, format!("[err] {}", line));
            }
        });
    }

    match child.wait().await {
        Ok(status) => {
            if status.success() {
                state.update_status(id, ProcessStatus::Completed(status.code().unwrap_or(0)));
            } else {
                state.update_status(
                    id,
                    ProcessStatus::Failed(format!("exit code {}", status.code().unwrap_or(-1))),
                );
            }
        }
        Err(e) => {
            state.update_status(id, ProcessStatus::Failed(e.to_string()));
        }
    }
}
