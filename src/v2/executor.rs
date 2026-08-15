use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

pub struct Executor {

}

impl Executor {
    pub async fn exec_blocking(script: &Path, module_path: &Path) -> Result<i32> {
        let status = Command::new("bash")
            .arg(script)
            .arg(module_path)
            .status()
            .await?;
        Ok(status.code().unwrap_or(1))
    }
}