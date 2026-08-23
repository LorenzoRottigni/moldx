use std::str::FromStr;
use std::fmt;

use crate::errors::MoldXError;

/// The kinds of entities managed by MoldX.
///
/// - [`Strategy`](Entity::Strategy) describes how a module can be processed.
/// - [`Template`](Entity::Template) defines the files used to identify
///   modules and strategies.
/// - [`Module`](Entity::Module) represents a discovered project module.
/// - [`Command`](Entity::Command) represents an executable strategy script.
#[derive(Debug)]
pub enum Entity {
    Strategy,
    Template,
    Module,
    Command,
    Profile
}

impl Entity {
    /// Returns the canonical lowercase name of the entity.
    ///
    /// # Returns
    ///
    /// The entity name as a static string, e.g. `"strategy"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Entity::Strategy => "strategy",
            Entity::Template => "template",
            Entity::Module => "module",
            Entity::Command => "command",
            Entity::Profile => "profile",
        }
    }
}

impl FromStr for Entity {
    type Err = MoldXError;

    /// Parses an entity name into an [`Entity`].
    ///
    /// # Arguments
    ///
    /// * `value` - The entity name, e.g. `"strategy"`.
    ///
    /// # Returns
    ///
    /// The corresponding [`Entity`] variant.
    ///
    /// # Errors
    ///
    /// Returns [`MoldXError::UnknownEntity`] if the value does not name a
    /// known entity.
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
            Entity::Profile => write!(f, "profile")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str() {
        assert_eq!(Entity::Strategy.as_str(), "strategy");
        assert_eq!(Entity::Template.as_str(), "template");
        assert_eq!(Entity::Module.as_str(), "module");
        assert_eq!(Entity::Command.as_str(), "command");
    }

    #[test]
    fn test_from_str_valid() {
        assert!(matches!("strategy".parse::<Entity>().unwrap(), Entity::Strategy));
        assert!(matches!("template".parse::<Entity>().unwrap(), Entity::Template));
        assert!(matches!("module".parse::<Entity>().unwrap(), Entity::Module));
        assert!(matches!("command".parse::<Entity>().unwrap(), Entity::Command));
    }

    #[test]
    fn test_from_str_invalid() {
        let result = "unknown".parse::<Entity>();
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(Entity::Strategy.to_string(), "strategy");
        assert_eq!(Entity::Template.to_string(), "template");
        assert_eq!(Entity::Module.to_string(), "module");
        assert_eq!(Entity::Command.to_string(), "command");
    }
}