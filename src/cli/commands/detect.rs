use std::path::PathBuf;

use crate::client::MoldXClient;
use anyhow::Result;

/// Prints the profiles detected for a given path.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `path` - The module path whose profiles should be detected.
///
/// # Returns
///
/// Ok after printing the detected profiles.
///
/// # Errors
///
/// Always returns `Ok(())`.
pub async fn detect(
    client: &MoldXClient,
    path: PathBuf,
) -> Result<()> {
    let profiles = client.profiles_for_module(&path);
    println!("Detected profiles for {}:", path.to_string_lossy());
    for p in &profiles {
        println!("  - {}", p.name);
    }
    Ok(())
}
