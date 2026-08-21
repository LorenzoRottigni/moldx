
use crate::client::MoldXClient;
use anyhow::Result;

/// Prints a snapshot of the resolved MoldX project.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
///
/// # Returns
///
/// Ok after printing the snapshot.
///
/// # Errors
///
/// Always returns `Ok(())`.
pub async fn list(client: &MoldXClient) -> Result<()> {
    println!("{}", client);
    Ok(())
}
