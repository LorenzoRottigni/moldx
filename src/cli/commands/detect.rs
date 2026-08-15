use std::path::PathBuf;

use crate::client::MoldXClient;
use anyhow::Result;

pub async fn detect(
    client: &MoldXClient,
    path: PathBuf
) -> Result<()> {
    let strategies = client.strategies_for_module(&path);
    println!("Detected strategies for {}:", path.to_string_lossy());
    for s in &strategies {
        println!("  - {}", s);
    }
    Ok(())
}
