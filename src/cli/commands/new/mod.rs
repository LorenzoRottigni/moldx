mod command;
mod module;
mod strategy;
mod template;

use crate::{client::MoldXClient, types::Entity, errors::MoldXError};
use anyhow::Result;

pub async fn new(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(MoldXError::NewUsage.into());
    }

    let entity = args[0]
        .parse::<Entity>()?;

    match entity {
        Entity::Strategy => strategy::new_strategy(client, args)?,
        Entity::Template => template::new_template(client, args)?,
        Entity::Module => module::new_module(client, args)?,
        Entity::Command => command::new_command(client, args)?
    }

    Ok(())
}
