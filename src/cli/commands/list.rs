//! `moldx list` subcommand.
//!
//! Prints the current MoldX state: configuration, executor status, profiles,
//! and modules.

use anyhow::Result;

use crate::client::MoldXClient;

/// Prints the current MoldX state for the resolved project.
pub async fn list(client: &MoldXClient) -> Result<()> {
    println!("{}", client);
    Ok(())
}
