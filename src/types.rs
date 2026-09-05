//! Shared type definitions.
//!
//! [`crate::types::Entity`] enumerates the kinds of artifacts MoldX manages: templates,
//! modules, commands, and profiles.

use crate::errors::MoldXError2;
use anyhow::Result;
use std::fmt;
use std::str::FromStr;

/// The kinds of entities managed by MoldX.
///
/// - [`Profile`](Entity::Profile) describes how a module can be processed.
/// - [`Template`](Entity::Template) defines the files used to identify
///   modules and profiles.
/// - [`Module`](Entity::Module) represents a discovered project module.
/// - [`Command`](Entity::Command) represents an executable profile script.
#[derive(Debug, Clone, Copy)]
pub enum Entity {
    Template,
    Module,
    Command,
    Profile,
}

impl Entity {
    /// Returns the canonical lowercase name of the entity.
    ///
    /// # Returns
    ///
    /// The entity name as a static string, e.g. `"profile"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Entity::Template => "template",
            Entity::Module => "module",
            Entity::Command => "command",
            Entity::Profile => "profile",
        }
    }
}

impl FromStr for Entity {
    type Err = MoldXError2;

    /// Parses an entity name into an [`Entity`].
    ///
    /// # Arguments
    ///
    /// * `value` - The entity name, e.g. `"profile"`.
    ///
    /// # Returns
    ///
    /// The corresponding [`Entity`] variant.
    ///
    /// # Errors
    ///
    /// Returns [`MoldXError2::UnknownEntity`] if the value does not name a
    /// known entity.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "template" => Ok(Entity::Template),
            "module" => Ok(Entity::Module),
            "command" => Ok(Entity::Command),
            "profile" => Ok(Entity::Profile),
            _ => Err(MoldXError2::UnknownEntity {
                entity: value.to_string(),
            }),
        }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::Template => write!(f, "template"),
            Entity::Module => write!(f, "module"),
            Entity::Command => write!(f, "command"),
            Entity::Profile => write!(f, "profile"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str() {
        assert_eq!(Entity::Template.as_str(), "template");
        assert_eq!(Entity::Module.as_str(), "module");
        assert_eq!(Entity::Command.as_str(), "command");
        assert_eq!(Entity::Profile.as_str(), "profile");
    }

    #[test]
    fn test_from_str_valid() {
        assert!(matches!(
            "template".parse::<Entity>().unwrap(),
            Entity::Template
        ));
        assert!(matches!(
            "module".parse::<Entity>().unwrap(),
            Entity::Module
        ));
        assert!(matches!(
            "command".parse::<Entity>().unwrap(),
            Entity::Command
        ));
        assert!(matches!(
            "profile".parse::<Entity>().unwrap(),
            Entity::Profile
        ));
    }

    #[test]
    fn test_from_str_invalid() {
        let result = "unknown".parse::<Entity>();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_rejects_unknown_entity() {
        let result = "strategy".parse::<Entity>();
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(Entity::Template.to_string(), "template");
        assert_eq!(Entity::Module.to_string(), "module");
        assert_eq!(Entity::Command.to_string(), "command");
        assert_eq!(Entity::Profile.to_string(), "profile");
    }
}
