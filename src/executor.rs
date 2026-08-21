use anyhow::Result;
use owo_colors::OwoColorize;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display},
    path::Path,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tokio::process::{Child, Command};

use crate::errors::MoldXError;

type PID = u32;

/// Status of a tracked process.
///
/// - [`Running`](ProcessStatus::Running) means the process is still executing.
/// - [`Completed`](ProcessStatus::Completed) carries the exit code.
/// - [`Failed`](ProcessStatus::Failed) carries a failure description.
/// - [`Killed`](ProcessStatus::Killed) means the process was terminated.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Completed(i32),
    Failed(String),
    Killed,
}

impl ProcessStatus {
    /// Returns a plain-text label for the status.
    ///
    /// # Returns
    ///
    /// A human-readable description such as `"Running"` or `"Done(0)"`.
    pub fn label(&self) -> String {
        match self {
            ProcessStatus::Running => "Running".to_string(),
            ProcessStatus::Completed(code) => format!("Done({})", code),
            ProcessStatus::Failed(msg) => format!("Failed: {}", msg),
            ProcessStatus::Killed => "Killed".to_string(),
        }
    }

    /// Returns whether the process is still running.
    ///
    /// # Returns
    ///
    /// `true` if the status is [`ProcessStatus::Running`].
    pub fn is_running(&self) -> bool {
        matches!(self, ProcessStatus::Running)
    }
}

impl Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "{}", "Running".yellow()),
            ProcessStatus::Completed(code) => write!(f, "{}", format!("Done({})", code).green()),
            ProcessStatus::Failed(msg) => write!(f, "{}", format!("Failed: {}", msg).red()),
            ProcessStatus::Killed => write!(f, "{}", "Killed".red()),
        }
    }
}

/// A tracked process with its accumulated output.
///
/// `RunningProcess` stores the metadata needed to identify and manage a
/// spawned script together with a bounded buffer of its output lines.
#[derive(Debug, Clone)]
pub struct RunningProcess {
    pub id: u64,
    pub module_path: String,
    pub strategy: String,
    pub command: String,
    pub pid: Option<u32>,
    pub started_at: SystemTime,
    pub status: ProcessStatus,
    pub output_lines: VecDeque<String>,
}

/// An immutable snapshot of a tracked process.
///
/// Unlike [`RunningProcess`], a summary omits the buffered output lines,
/// making it cheap to query for display purposes.
#[derive(Debug, Clone)]
pub struct ProcessSummary {
    pub id: u64,
    pub module_path: String,
    pub strategy: String,
    pub command: String,
    pub pid: Option<u32>,
    pub started_at: SystemTime,
    pub status: ProcessStatus,
}

#[derive(Debug, Default)]
struct state {
    processes: Vec<RunningProcess>,
    next_id: u64,
}

/// Handles command execution and process tracking.
///
/// `Executor` spawns strategy scripts both in the background and in
/// blocking mode, and keeps shared state for tracked processes so their
/// status and output can be inspected from other tasks.
#[derive(Debug)]
pub struct Executor {
    processes: HashMap<PID, Child>,
    state: Arc<Mutex<state>>,
}

impl Clone for Executor {
    fn clone(&self) -> Self {
        Self {
            processes: HashMap::new(),
            state: Arc::clone(&self.state),
        }
    }
}

impl Executor {
    /// Creates a new executor with no tracked processes.
    ///
    /// # Returns
    ///
    /// An empty [`Executor`].
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            state: Arc::new(Mutex::new(state {
                processes: Vec::new(),
                next_id: 0,
            })),
        }
    }

    /// Spawns a script in the background and registers it by PID.
    ///
    /// The script is executed with `bash` and receives the module path as
    /// its first argument. The spawned child is not tracked in the shared
    /// process state; use [`run_and_track`] for tracked runs.
    ///
    /// # Arguments
    ///
    /// * `script` - Path to the shell script to execute.
    /// * `module_path` - Path passed to the script as its first argument.
    ///
    /// # Returns
    ///
    /// The PID of the spawned process.
    ///
    /// # Errors
    ///
    /// Returns [`MoldXError::ProcessSpawnFailed`] if the process cannot be
    /// spawned or its PID cannot be determined.
    pub async fn exec(
        &mut self,
        script: &Path,
        module_path: &Path,
    ) -> Result<u32> {
        let child = Command::new("bash")
            .arg(script)
            .arg(module_path)
            .spawn()
            .map_err(|e| MoldXError::ProcessSpawnFailed { reason: e.to_string() })?;

        let pid = child.id().ok_or_else(|| MoldXError::ProcessSpawnFailed { reason: "failed to get process ID".to_string() })?;
        self.processes.insert(pid, child);
        Ok(pid)
    }

    /// Runs a script to completion and returns its exit code.
    ///
    /// The script is executed with `bash` and receives the module path as
    /// its first argument.
    ///
    /// # Arguments
    ///
    /// * `script` - Path to the shell script to execute.
    /// * `module_path` - Path passed to the script as its first argument.
    ///
    /// # Returns
    ///
    /// The exit code of the script, or `1` if the code cannot be determined.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be spawned or waited on.
    pub async fn exec_blocking(
        &self,
        script: &Path,
        module_path: &Path,
    ) -> Result<i32> {
        let status = Command::new("bash")
            .arg(script)
            .arg(module_path)
            .status()
            .await?;

        Ok(status.code().unwrap_or(1))
    }

    // State management methods

    /// Registers a new tracked process and assigns it a unique ID.
    ///
    /// # Arguments
    ///
    /// * `module_path` - Path of the module the process runs against.
    /// * `strategy` - Name of the strategy being executed.
    /// * `command` - Name of the command being executed.
    /// * `pid` - Operating system PID, if already known.
    ///
    /// # Returns
    ///
    /// The unique identifier assigned to the tracked process.
    pub fn add_process(&self, module_path: &str, strategy: &str, command: &str, pid: Option<u32>) -> u64 {
        let mut g = self.state.lock().unwrap();
        let id = g.next_id;
        g.next_id += 1;
        g.processes.push(RunningProcess {
            id,
            module_path: module_path.to_string(),
            strategy: strategy.to_string(),
            command: command.to_string(),
            pid,
            started_at: SystemTime::now(),
            status: ProcessStatus::Running,
            output_lines: VecDeque::new(),
        });
        id
    }

    /// Updates the operating system PID of a tracked process.
    ///
    /// Does nothing if no process with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - Identifier of the tracked process.
    /// * `pid` - The new PID, or `None` to clear it.
    pub fn update_pid(&self, id: u64, pid: Option<u32>) {
        let mut g = self.state.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.pid = pid;
        }
    }

    /// Updates the status of a tracked process.
    ///
    /// Does nothing if no process with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - Identifier of the tracked process.
    /// * `status` - The new [`ProcessStatus`].
    pub fn update_status(&self, id: u64, status: ProcessStatus) {
        let mut g = self.state.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.status = status;
        }
    }

    /// Appends an output line to a tracked process.
    ///
    /// The buffer is capped at 500 lines; older lines are dropped. Does
    /// nothing if no process with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - Identifier of the tracked process.
    /// * `line` - The output line to append.
    pub fn append_output(&self, id: u64, line: String) {
        let mut g = self.state.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.output_lines.push_back(line);
            if p.output_lines.len() > 500 {
                p.output_lines.pop_front();
            }
        }
    }

    /// Returns snapshots of all tracked processes.
    ///
    /// # Returns
    ///
    /// A [`ProcessSummary`] for every tracked process, in registration order.
    pub fn get_summaries(&self) -> Vec<ProcessSummary> {
        let g = self.state.lock().unwrap();
        g.processes
            .iter()
            .map(|p| ProcessSummary {
                id: p.id,
                module_path: p.module_path.clone(),
                strategy: p.strategy.clone(),
                command: p.command.clone(),
                pid: p.pid,
                started_at: p.started_at,
                status: p.status.clone(),
            })
            .collect()
    }

    /// Returns the buffered output lines of a tracked process.
    ///
    /// # Arguments
    ///
    /// * `id` - Identifier of the tracked process.
    ///
    /// # Returns
    ///
    /// The process output lines, or an empty queue if the process is unknown.
    pub fn get_output(&self, id: u64) -> VecDeque<String> {
        let g = self.state.lock().unwrap();
        g.processes
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.output_lines.clone())
            .unwrap_or_default()
    }

    /// Terminates a tracked process and marks it as killed.
    ///
    /// On Unix, the whole process group receives `SIGTERM`. Processes
    /// without a known PID are still marked as killed.
    ///
    /// # Arguments
    ///
    /// * `id` - Identifier of the tracked process.
    pub fn kill_process(&self, id: u64) {
        let pid = {
            let g = self.state.lock().unwrap();
            g.processes.iter().find(|p| p.id == id).and_then(|p| p.pid)
        };

        if let Some(pid) = pid {
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-TERM", &format!("-{}", pid)])
                    .status();
            }
            #[cfg(not(unix))]
            {
                let _ = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .status();
            }
        }

        self.update_status(id, ProcessStatus::Killed);
    }

    /// Kills all tracked processes that are still running.
    pub fn kill_all_running(&self) {
        let running_ids: Vec<u64> = {
            let g = self.state.lock().unwrap();
            g.processes
                .iter()
                .filter(|p| p.status.is_running())
                .map(|p| p.id)
                .collect()
        };

        for id in running_ids {
            self.kill_process(id);
        }
    }
}

impl Display for Executor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let g = self.state.lock().map_err(|_| fmt::Error)?;
        writeln!(f, "{} {}", "executor:".bold().yellow(), format!("{} running", g.processes.len()))?;
        for process in &g.processes {
            writeln!(
                f,
                "  {} {} · {}/{} · pid={:?} · {}",
                format!("#{}", process.id).dimmed(),
                process.module_path.green(),
                process.strategy.cyan(),
                process.command.cyan(),
                process.pid,
                process.status
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_status_label() {
        assert_eq!(ProcessStatus::Running.label(), "Running");
        assert_eq!(ProcessStatus::Completed(0).label(), "Done(0)");
        assert_eq!(ProcessStatus::Completed(1).label(), "Done(1)");
        assert_eq!(ProcessStatus::Failed("timeout".into()).label(), "Failed: timeout");
        assert_eq!(ProcessStatus::Killed.label(), "Killed");
    }

    #[test]
    fn test_process_status_is_running() {
        assert!(ProcessStatus::Running.is_running());
        assert!(!ProcessStatus::Completed(0).is_running());
        assert!(!ProcessStatus::Failed("x".into()).is_running());
        assert!(!ProcessStatus::Killed.is_running());
    }

    #[test]
    fn test_process_status_display() {
        let display = ProcessStatus::Running.to_string();
        assert!(display.contains("Running"));
        let display = ProcessStatus::Completed(0).to_string();
        assert!(display.contains("Done(0)"));
        let display = ProcessStatus::Failed("err".into()).to_string();
        assert!(display.contains("Failed: err"));
        let display = ProcessStatus::Killed.to_string();
        assert!(display.contains("Killed"));
    }

    #[test]
    fn test_executor_new() {
        let executor = Executor::new();
        assert!(executor.processes.is_empty());
        assert!(executor.get_summaries().is_empty());
    }

    #[test]
    fn test_executor_clone() {
        let executor = Executor::new();
        let cloned = executor.clone();
        assert!(cloned.processes.is_empty());
    }

    #[test]
    fn test_add_process() {
        let executor = Executor::new();
        let id = executor.add_process("/path/to/module", "docker", "build", Some(1234));
        assert_eq!(id, 0);
        let summaries = executor.get_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].module_path, "/path/to/module");
        assert_eq!(summaries[0].strategy, "docker");
        assert_eq!(summaries[0].command, "build");
        assert_eq!(summaries[0].pid, Some(1234));
        assert!(summaries[0].status.is_running());
    }

    #[test]
    fn test_add_process_increments_id() {
        let executor = Executor::new();
        let id1 = executor.add_process("a", "s", "c", None);
        let id2 = executor.add_process("b", "s", "c", None);
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(executor.get_summaries().len(), 2);
    }

    #[test]
    fn test_update_pid() {
        let executor = Executor::new();
        let id = executor.add_process("/m", "s", "c", None);
        executor.update_pid(id, Some(5678));
        assert_eq!(executor.get_summaries()[0].pid, Some(5678));
    }

    #[test]
    fn test_update_pid_nonexistent() {
        let executor = Executor::new();
        executor.update_pid(999, Some(1));
    }

    #[test]
    fn test_update_status() {
        let executor = Executor::new();
        let id = executor.add_process("/m", "s", "c", None);
        executor.update_status(id, ProcessStatus::Completed(0));
        assert!(!executor.get_summaries()[0].status.is_running());
        assert_eq!(executor.get_summaries()[0].status.label(), "Done(0)");
    }

    #[test]
    fn test_update_status_nonexistent() {
        let executor = Executor::new();
        executor.update_status(999, ProcessStatus::Killed);
    }

    #[test]
    fn test_append_output() {
        let executor = Executor::new();
        let id = executor.add_process("/m", "s", "c", None);
        executor.append_output(id, "line1".into());
        executor.append_output(id, "line2".into());
        let output = executor.get_output(id);
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], "line1");
        assert_eq!(output[1], "line2");
    }

    #[test]
    fn test_append_output_500_cap() {
        let executor = Executor::new();
        let id = executor.add_process("/m", "s", "c", None);
        for i in 0..600 {
            executor.append_output(id, format!("line{}", i));
        }
        let output = executor.get_output(id);
        assert_eq!(output.len(), 500);
        assert_eq!(output[0], "line100");
        assert_eq!(output[499], "line599");
    }

    #[test]
    fn test_append_output_nonexistent() {
        let executor = Executor::new();
        executor.append_output(999, "x".into());
    }

    #[test]
    fn test_get_output_nonexistent() {
        let executor = Executor::new();
        let output = executor.get_output(999);
        assert!(output.is_empty());
    }

    #[test]
    fn test_get_summaries_empty() {
        let executor = Executor::new();
        assert!(executor.get_summaries().is_empty());
    }

    #[test]
    fn test_kill_process() {
        let executor = Executor::new();
        let id = executor.add_process("/m", "s", "c", Some(99999));
        executor.kill_process(id);
        assert_eq!(executor.get_summaries()[0].status, ProcessStatus::Killed);
    }

    #[test]
    fn test_kill_process_no_pid() {
        let executor = Executor::new();
        let id = executor.add_process("/m", "s", "c", None);
        executor.kill_process(id);
        assert_eq!(executor.get_summaries()[0].status, ProcessStatus::Killed);
    }

    #[test]
    fn test_kill_all_running() {
        let executor = Executor::new();
        let id1 = executor.add_process("/m1", "s", "c", None);
        let id2 = executor.add_process("/m2", "s", "c", None);
        let id3 = executor.add_process("/m3", "s", "c", None);
        executor.update_status(id2, ProcessStatus::Completed(0));
        executor.kill_all_running();
        let summaries = executor.get_summaries();
        assert_eq!(summaries.iter().find(|s| s.id == id1).unwrap().status, ProcessStatus::Killed);
        assert!(summaries.iter().find(|s| s.id == id2).unwrap().status.is_running() == false);
        assert_eq!(summaries.iter().find(|s| s.id == id3).unwrap().status, ProcessStatus::Killed);
    }

    #[test]
    fn test_executor_display() {
        let executor = Executor::new();
        let _id = executor.add_process("/m", "s", "c", Some(123));
        let display = executor.to_string();
        assert!(display.contains("1 running"));
        assert!(display.contains("/m"));
    }

    #[test]
    fn test_executor_display_empty() {
        let executor = Executor::new();
        let display = executor.to_string();
        assert!(display.contains("0 running"));
    }

    #[tokio::test]
    async fn test_exec_blocking_success() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("test.sh");
        std::fs::write(&script, "#!/bin/bash\nexit 0").unwrap();
        let executor = Executor::new();
        let code = executor.exec_blocking(&script, dir.path()).await.unwrap();
        assert_eq!(code, 0);
    }

    #[tokio::test]
    async fn test_exec_blocking_failure() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("fail.sh");
        std::fs::write(&script, "#!/bin/bash\nexit 42").unwrap();
        let executor = Executor::new();
        let code = executor.exec_blocking(&script, dir.path()).await.unwrap();
        assert_eq!(code, 42);
    }

    #[tokio::test]
    async fn test_exec_success() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("run.sh");
        std::fs::write(&script, "#!/bin/bash\nsleep 0.1").unwrap();
        let mut executor = Executor::new();
        let pid = executor.exec(&script, dir.path()).await.unwrap();
        assert!(pid > 0);
    }

    #[tokio::test]
    async fn test_exec_nonexistent_script() {
        let mut executor = Executor::new();
        let result = executor.exec(
            std::path::Path::new("/nonexistent/script.sh"),
            std::path::Path::new("/tmp"),
        ).await;
        if let Ok(pid) = result {
            assert!(pid > 0);
        }
    }

    #[tokio::test]
    async fn test_run_and_track_success() {
        use std::sync::Arc;
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("track.sh");
        std::fs::write(&script, "#!/bin/bash\necho hello").unwrap();
        let executor = Arc::new(Executor::new());
        let id = executor.add_process("/m", "s", "c", None);
        run_and_track(executor.clone(), id, script, dir.path().to_path_buf()).await;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let summary = &executor.get_summaries()[0];
        assert!(!summary.status.is_running());
        assert_eq!(summary.status.label(), "Done(0)");
        let output = executor.get_output(id);
        assert!(output.iter().any(|l| l.contains("hello")));
    }

    #[tokio::test]
    async fn test_run_and_track_failure() {
        use std::sync::Arc;
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("fail_track.sh");
        std::fs::write(&script, "#!/bin/bash\necho err >&2\nexit 1").unwrap();
        let executor = Arc::new(Executor::new());
        let id = executor.add_process("/m", "s", "c", None);
        run_and_track(executor.clone(), id, script, dir.path().to_path_buf()).await;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let summary = &executor.get_summaries()[0];
        assert!(!summary.status.is_running());
        let output = executor.get_output(id);
        assert!(output.iter().any(|l| l.contains("[err]")));
    }

    #[tokio::test]
    async fn test_run_and_track_spawn_failure() {
        use std::sync::Arc;
        let executor = Arc::new(Executor::new());
        let id = executor.add_process("/m", "s", "c", None);
        run_and_track(
            executor.clone(),
            id,
            std::path::PathBuf::from("/nonexistent/script.sh"),
            std::path::PathBuf::from("/tmp"),
        ).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let summary = &executor.get_summaries()[0];
        assert!(!summary.status.is_running());
    }
}

/// Spawns `script` in the background, streams its output into the executor,
/// and updates the process status when it exits.
///
/// Standard output lines are appended verbatim; standard error lines are
/// prefixed with `[err]`. On Unix the child runs in its own process group so
/// it can be killed as a whole.
///
/// # Arguments
///
/// * `executor` - Shared executor holding the tracked process state.
/// * `id` - Identifier of the tracked process, as returned by
///   [`Executor::add_process`].
/// * `script` - Path to the shell script to execute.
/// * `module_path` - Path passed to the script as its first argument.
pub async fn run_and_track(
    executor: Arc<Executor>,
    id: u64,
    script: std::path::PathBuf,
    module_path: std::path::PathBuf,
) {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut cmd = Command::new("bash");
    cmd.arg(&script)
        .arg(&module_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        cmd.process_group(0);
    }

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            executor.update_status(id, ProcessStatus::Failed(e.to_string()));
            return;
        }
    };

    let pid = child.id();
    executor.update_pid(id, pid);

    if let Some(stdout) = child.stdout.take() {
        let executor2 = executor.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                executor2.append_output(id, line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let executor2 = executor.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                executor2.append_output(id, format!("[err] {}", line));
            }
        });
    }

    match child.wait().await {
        Ok(status) => {
            if status.success() {
                executor.update_status(id, ProcessStatus::Completed(status.code().unwrap_or(0)));
            } else {
                executor.update_status(
                    id,
                    ProcessStatus::Failed(format!("exit code {}", status.code().unwrap_or(-1))),
                );
            }
        }
        Err(e) => {
            executor.update_status(id, ProcessStatus::Failed(e.to_string()));
        }
    }
}
