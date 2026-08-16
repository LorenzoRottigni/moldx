
use crate::client::MoldXClient;
use anyhow::Result;

pub async fn list(client: &MoldXClient) -> Result<()> {
    println!("{}", client);
    Ok(())
}
