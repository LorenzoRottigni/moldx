use anyhow::Result;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display},
    path::Path,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tokio::process::{Child, Command};

type PID = u32;

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Running,
    Completed(i32),
    Failed(String),
    Killed,
}

impl ProcessStatus {
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

impl Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

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
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            state: Arc::new(Mutex::new(state {
                processes: Vec::new(),
                next_id: 0,
            })),
        }
    }

    pub async fn exec(
        &mut self,
        script: &Path,
        module_path: &Path,
    ) -> Result<u32> {
        let child = Command::new("bash")
            .arg(script)
            .arg(module_path)
            .spawn()?;

        let pid = child.id().unwrap();
        self.processes.insert(pid, child);
        Ok(pid)
    }

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

    pub fn update_pid(&self, id: u64, pid: Option<u32>) {
        let mut g = self.state.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.pid = pid;
        }
    }

    pub fn update_status(&self, id: u64, status: ProcessStatus) {
        let mut g = self.state.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.status = status;
        }
    }

    pub fn append_output(&self, id: u64, line: String) {
        let mut g = self.state.lock().unwrap();
        if let Some(p) = g.processes.iter_mut().find(|p| p.id == id) {
            p.output_lines.push_back(line);
            if p.output_lines.len() > 500 {
                p.output_lines.pop_front();
            }
        }
    }

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

    pub fn get_output(&self, id: u64) -> VecDeque<String> {
        let g = self.state.lock().unwrap();
        g.processes
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.output_lines.clone())
            .unwrap_or_default()
    }

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
        writeln!(f, "Executor(processes: {})", g.processes.len())?;

        for process in &g.processes {
            writeln!(
                f,
                "  - #{} {} [{}] {} pid={:?} status={}",
                process.id,
                process.module_path,
                process.strategy,
                process.command,
                process.pid,
                process.status
            )?;
        }

        Ok(())
    }
}

/// Spawn `script` in the background, stream its output into executor, and
/// update the process status when it exits.
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
