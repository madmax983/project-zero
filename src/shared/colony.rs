//! Colony identity resource.

use bevy_ecs::prelude::*;

/// The name of the player's colony, generated at world creation.
#[derive(Resource, Debug, Clone)]
pub struct ColonyName {
    /// The colony's name.
    pub name: String,
}

impl Default for ColonyName {
    fn default() -> Self {
        Self {
            name: "The Colony".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colony_name_default() {
        let colony = ColonyName::default();
        assert_eq!(colony.name, "The Colony");
    }

    #[test]
    fn test_colony_name_custom() {
        let colony = ColonyName {
            name: "Nova Prime".to_string(),
        };
        assert_eq!(colony.name, "Nova Prime");
    }
}
