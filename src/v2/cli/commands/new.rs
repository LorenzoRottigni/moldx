use crate::v2::client::MoldXClient;
use anyhow::Result;

pub async fn new(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    // moldx new strategy => moldx new <strategy>
    // moldx new template => moldx new [strategy] <template>
    // moldx new module => moldx new [strategy] [template] <module-path>
    // moldx new command => moldx new [strategy] <command>
    Ok(())
}