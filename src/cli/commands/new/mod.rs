mod command;
mod module;
mod strategy;
mod template;

use crate::{client::MoldXClient, types::Entity};
use anyhow::{anyhow, bail, Result};

pub async fn new(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: moldx new <strategy|template|module|command> ...");
    }

    let entity = args[0]
        .parse::<Entity>()
        .map_err(|err| anyhow!(err))?;

    match entity {
        Entity::Strategy => strategy::new_strategy(client, args)?,
        Entity::Template => template::new_template(client, args)?,
        Entity::Module => module::new_module(args)?,
        Entity::Command => command::new_command(client, args)?
    }

    Ok(())
}