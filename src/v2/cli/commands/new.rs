use crate::v2::{client::MoldXClient, types::Entity};
use anyhow::{Result, bail};

pub async fn new(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        bail!("")
    }

    if args.len() > 3 {
        bail!("")
    }

    let entity = args.get(0).expect("No entity arg provided").parse::<Entity>().expect("Unable to parse MoldX entity from provided args");

    match entity {
        Entity::Strategy => {
            // moldx new strategy <strategy>
        }
        Entity::Template => {
            // moldx new template [strategy] <template>
        }
        Entity::Module => {
            // moldx new module [strategy] [template] <module-path>
        }
        Entity::Command => {
            // moldx new command [strategy] <command>
        }
    }

    Ok(())
}