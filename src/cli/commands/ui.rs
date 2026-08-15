use crate::{client::MoldXClient, tui};
use anyhow::Result;

pub async fn ui(client: &MoldXClient) -> Result<()> {
    tui::run(client).await
}