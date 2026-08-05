//! Process registry used by the TUI runtime.
//!
//! [`AppState`] is a cheaply-cloneable handle backed by an `Arc<Mutex<_>>`.
//! All public methods acquire the mutex for the minimum time needed — no
//! `await` points occur while the lock is held, so deadlocks and starvation
//! are not possible.
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Lifecycle state of a tracked process.
#[derive(Debug, Clone)]
pub enum ProcessStatus {
    /// The process is still running.
    Running,
    /// The process exited with the given code.
    Completed(i32),
    /// The process exited with a non-zero code or could not be spawned.
    Failed(String),
    /// The process was killed via [`AppState::kill_process`].
    Killed,
}

impl ProcessStatus {
    /// Human-readable one-word label shown in the TUI and web UI.
    pub fn label(&self) -> String {
        match self {
            ProcessStatus::Running => "Running".to_string(),
            ProcessStatus::Completed(code) => format!("Done({})", code),
            ProcessStatus::Failed(msg) => format!("Failed: {}", msg),
            ProcessStatus::Killed => "Killed".to_string(),
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, ProcessStatus::Running)
    }
}

/// A single entry in the process registry.
#[derive(Debug, Clone)]
pub struct RunningProcess {
    /// Monotonically increasing identifier assigned at spawn time.
    pub id: u64,
    /// Absolute path of the module the command was run against.
    pub module_path: String,
    /// Strategy that was selected (e.g. `"docker"`).
    pub strategy: String,
    /// Command that was run (e.g. `"build"`).
    pub command: String,
    /// OS process ID of the spawned bash interpreter.
    pub pid: Option<u32>,
    /// Wall-clock time at which the process was registered.
    pub started_at: SystemTime,
    /// Current lifecycle state.
    pub status: ProcessStatus,
    /// Captured stdout/stderr lines, bounded to the last 500 entries.
    /// `VecDeque` gives O(1) front removal when the buffer is full.
    pub output_lines: VecDeque<String>,
}

/// Minimal process info returned by [`AppState::get_summaries`].
///
/// Does not include `output_lines` so the per-tick TUI render never clones
/// output for processes that are not on screen.
#[derive(Debug, Clone)]
#[allow(dead_code)] // public fields kept as stable API for external consumers
pub struct ProcessSummary {
    pub id: u64,
    pub module_path: String,
    pub strategy: String,
    pub command: String,
    pub pid: Option<u32>,
    pub started_at: SystemTime,
    pub status: ProcessStatus,
}

// ─── Internal storage ────────────────────────────────────────────────────────

/// Internal storage; never exposed outside this module.
#[derive(Default)]
struct Inner {
    processes: Vec<RunningProcess>,
    next_id: u64,
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Thread-safe process registry shared across the TUI runtime.
///
/// Cloning an [`AppState`] is `O(1)` — it just increments an `Arc` reference
/// count.  All clones share the same underlying data.
#[derive(Clone, Default)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

impl AppState {
    // `new` is an alias for `Default::default()`, kept as a conventional constructor.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new process and return its assigned `id`.
    pub fn add_process(
        &self,
        module_path: &str,
        strategy: &str,
        command: &str,
        pid: Option<u32>,
    ) -> u64 {
        let mut g = self.inner.lock().unwrap();
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

    /// Update the PID once the OS process has been spawned.
    pub fn update_pid(&self, id: u64, pid: Option<u32>) {
        {
            let mut g = self.inner.lock().unwrap();
            if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
                p.pid = pid;
            }
        }
    }

    /// Transition the process to a terminal or updated status.
    pub fn update_status(&self, id: u64, status: ProcessStatus) {
        {
            let mut g = self.inner.lock().unwrap();
            if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
                p.status = status;
            }
        }
    }

    /// Append a captured output line, dropping the oldest if the buffer exceeds 500 lines.
    pub fn append_output(&self, id: u64, line: String) {
        let mut g = self.inner.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.output_lines.push_back(line);
            if p.output_lines.len() > 500 {
                p.output_lines.pop_front(); // O(1) with VecDeque
            }
        }
    }

    /// Return a snapshot of all registered processes including their output.
    ///
    /// Prefer [`get_summaries`] + [`get_output`] in hot render paths to avoid
    /// cloning output for every process on every tick.
    #[allow(dead_code)]
    pub fn get_all(&self) -> Vec<RunningProcess> {
        self.inner.lock().unwrap().processes.clone()
    }

    /// Return process metadata for all registered processes, without output lines.
    ///
    /// Use this in render loops where you only need status/pid/name — it avoids
    /// cloning potentially large output buffers for processes not on screen.
    pub fn get_summaries(&self) -> Vec<ProcessSummary> {
        let g = self.inner.lock().unwrap();
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

    /// Clone the output buffer for a single process.  Returns an empty deque if
    /// `id` is not found.
    pub fn get_output(&self, id: u64) -> VecDeque<String> {
        let g = self.inner.lock().unwrap();
        g.processes
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.output_lines.clone())
            .unwrap_or_default()
    }

    /// Send SIGTERM to the process group of `id` and mark it as [`ProcessStatus::Killed`].
    ///
    /// On Unix the negative-PID form of `kill` is used so that all child
    /// processes spawned by the script also receive the signal.  On non-Unix
    /// platforms only the top-level PID is targeted.
    pub fn kill_process(&self, id: u64) {
        let pid = {
            let g = self.inner.lock().unwrap();
            g.processes.iter().find(|p| p.id == id).and_then(|p| p.pid)
        };
        if let Some(pid) = pid {
            #[cfg(unix)]
            {
                // Negative PID sends SIGTERM to the entire process group (PGID == PID via process_group(0))
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

    /// Kill every process that is currently marked as running.
    ///
    /// This is used by the TUI shutdown path so closing the interface leaves
    /// no tracked child processes behind.
    pub fn kill_all_running(&self) {
        let running_ids: Vec<u64> = {
            let g = self.inner.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_process_assigns_incrementing_ids() {
        let state = AppState::new();
        let id0 = state.add_process("/mod/a", "docker", "build", None);
        let id1 = state.add_process("/mod/b", "node", "test", None);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn get_all_returns_all_processes() {
        let state = AppState::new();
        state.add_process("/mod/a", "docker", "build", None);
        state.add_process("/mod/b", "node", "start", None);
        let all = state.get_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn new_process_starts_as_running() {
        let state = AppState::new();
        let id = state.add_process("/mod", "rust", "build", None);
        let all = state.get_all();
        let p = all.iter().find(|p| p.id == id).unwrap();
        assert!(matches!(p.status, ProcessStatus::Running));
    }

    #[test]
    fn update_status_to_completed() {
        let state = AppState::new();
        let id = state.add_process("/mod", "docker", "build", None);
        state.update_status(id, ProcessStatus::Completed(0));
        let all = state.get_all();
        assert!(matches!(all[0].status, ProcessStatus::Completed(0)));
    }

    #[test]
    fn update_pid_stores_pid() {
        let state = AppState::new();
        let id = state.add_process("/mod", "docker", "build", None);
        state.update_pid(id, Some(12345));
        let all = state.get_all();
        assert_eq!(all[0].pid, Some(12345));
    }

    #[test]
    fn append_output_grows_buffer() {
        let state = AppState::new();
        let id = state.add_process("/mod", "docker", "build", None);
        state.append_output(id, "line 1".to_string());
        state.append_output(id, "line 2".to_string());
        let all = state.get_all();
        assert_eq!(
            all[0].output_lines.iter().collect::<Vec<_>>(),
            vec!["line 1", "line 2"]
        );
    }

    #[test]
    fn append_output_is_bounded_to_500_lines() {
        let state = AppState::new();
        let id = state.add_process("/mod", "docker", "build", None);
        for i in 0..600 {
            state.append_output(id, format!("line {}", i));
        }
        let all = state.get_all();
        assert!(all[0].output_lines.len() <= 500);
        assert_eq!(all[0].output_lines.back().unwrap(), "line 599");
    }

    #[test]
    fn get_summaries_excludes_output_lines() {
        let state = AppState::new();
        let id = state.add_process("/mod", "docker", "build", None);
        for i in 0..10 {
            state.append_output(id, format!("line {}", i));
        }
        let summaries = state.get_summaries();
        assert_eq!(summaries.len(), 1);
        // get_summaries does not carry output; get_output fetches it separately
        let output = state.get_output(summaries[0].id);
        assert_eq!(output.len(), 10);
    }

    #[test]
    fn process_status_label() {
        assert_eq!(ProcessStatus::Running.label(), "Running");
        assert_eq!(ProcessStatus::Completed(0).label(), "Done(0)");
        assert!(
            ProcessStatus::Failed("timeout".into())
                .label()
                .contains("timeout")
        );
        assert_eq!(ProcessStatus::Killed.label(), "Killed");
    }

    #[test]
    fn process_status_is_running() {
        assert!(ProcessStatus::Running.is_running());
        assert!(!ProcessStatus::Completed(0).is_running());
        assert!(!ProcessStatus::Killed.is_running());
    }

    #[test]
    fn update_status_on_unknown_id_is_noop() {
        let state = AppState::new();
        state.update_status(999, ProcessStatus::Killed); // should not panic
        assert!(state.get_all().is_empty());
    }

    #[test]
    fn clone_shares_inner_state() {
        let state = AppState::new();
        let clone = state.clone();
        let id = state.add_process("/mod", "docker", "build", None);
        let all = clone.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, id);
    }
}
