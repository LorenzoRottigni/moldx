use std::str::FromStr;
use std::fmt;

use crate::errors::MoldXError;

#[derive(Debug)]
pub enum Entity {
    Strategy,
    Template,
    Module,
    Command,
}

impl Entity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Entity::Strategy => "strategy",
            Entity::Template => "template",
            Entity::Module => "module",
            Entity::Command => "command",
        }
    }
}

impl FromStr for Entity {
    type Err = MoldXError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "strategy" => Ok(Entity::Strategy),
            "template" => Ok(Entity::Template),
            "module" => Ok(Entity::Module),
            "command" => Ok(Entity::Command),
            _ => Err(MoldXError::UnknownEntity { entity: value.to_string() }),
        }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::Strategy => write!(f, "strategy"),
            Entity::Template => write!(f, "template"),
            Entity::Module => write!(f, "module"),
            Entity::Command => write!(f, "command"),
        }
    }
}