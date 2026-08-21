use crate::{client::MoldXClient, tui};
use anyhow::Result;

/// Launches the interactive terminal UI.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
///
/// # Returns
///
/// Ok when the UI exits normally.
///
/// # Errors
///
/// Returns an error if the UI fails to run; see [`tui::run`].
pub async fn ui(client: &MoldXClient) -> Result<()> {
    tui::run(client).await
}