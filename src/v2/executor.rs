use anyhow::Result;
use std::{
    collections::HashMap,
    path::Path,
};
use tokio::process::{Child, Command};

type PID = u32;

#[derive(Debug)]
pub struct Executor {
    processes: HashMap<PID, Child>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
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
}