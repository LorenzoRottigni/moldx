use std::path::PathBuf;

use crate::client::MoldXClient;
use anyhow::Result;

/// Prints the strategies detected for a given path.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `path` - The module path whose strategies should be detected.
///
/// # Returns
///
/// Ok after printing the detected strategies.
///
/// # Errors
///
/// Always returns `Ok(())`.
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
