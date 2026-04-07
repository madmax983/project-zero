//! **Symbiotic Infrastructure** module.
//!
//! Introduces living buildings that integrate biological mechanics with standard colony infrastructure.
//! Symbiotic structures have environmental needs (like light and temperature) and can autonomously regenerate
//! damage over time, provided their needs are met.
//!
//! ## Mechanics
//! - **The Needs:** A [`SymbioticStructure`] relies on the `process_symbiotic_needs_system` to check if local environmental conditions meet its [`SymbioticNeeds`].
//! - **Dormancy:** If needs fall below required thresholds, the structure enters a dormant state to conserve energy.
//! - **Regeneration:** While active, the `process_symbiotic_regeneration_system` slowly repairs the building's core `Structure` health points.
//!

use crate::layer1::structure::Structure;
use bevy::prelude::*;

/// Defines a building that possesses living, biological traits.
///
/// # Examples
/// ```rust
/// use scale::layer1::symbiotic_infrastructure::SymbioticStructure;
///
/// let living_wall = SymbioticStructure {
///     regeneration_rate: 5.0,
///     is_dormant: false,
/// };
/// ```
#[derive(Component)]
pub struct SymbioticStructure {
    pub regeneration_rate: f32,
    pub is_dormant: bool,
}

/// The environmental thresholds required to keep a symbiotic structure active.
#[derive(Component)]
pub struct SymbioticNeeds {
    pub required_light: f32,
    pub required_temp: f32,
}

/// A simple representation of local environmental conditions.
///
/// *Note: This is currently used primarily as a test stub for integration.*
#[derive(Component)]
pub struct EnvironmentStatus {
    pub current_light: f32,
    pub current_temp: f32,
}

/// Evaluates whether a living building has the resources to remain active.
///
/// Compares the [`EnvironmentStatus`] against the [`SymbioticNeeds`] and toggles the `is_dormant` flag
/// on the [`SymbioticStructure`] accordingly.
pub fn process_symbiotic_needs_system(
    mut query: Query<(&mut SymbioticStructure, &SymbioticNeeds, &EnvironmentStatus)>,
) {
    for (mut symbiotic, needs, env) in query.iter_mut() {
        symbiotic.is_dormant =
            env.current_light < needs.required_light || env.current_temp < needs.required_temp;
    }
}

/// Applies biological healing to active living structures.
///
/// Restores `Structure` HP per tick based on the `regeneration_rate` of the [`SymbioticStructure`],
/// provided the entity is not dormant.
pub fn process_symbiotic_regeneration_system(
    mut query: Query<(&mut Structure, &SymbioticStructure)>,
) {
    for (mut structure, symbiotic) in query.iter_mut() {
        if !symbiotic.is_dormant {
            structure.current_hp += symbiotic.regeneration_rate;
            if structure.current_hp > structure.max_hp {
                structure.current_hp = structure.max_hp;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::structure::Structure;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                process_symbiotic_needs_system,
                process_symbiotic_regeneration_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn test_symbiotic_structure_regenerates_health() {
        let mut app = setup_app();

        // Spawn a damaged symbiotic structure
        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                SymbioticStructure {
                    regeneration_rate: 5.0,
                    is_dormant: false,
                },
            ))
            .id();

        app.update();

        // Structure HP should increase
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(
            structure.current_hp, 55.0,
            "Symbiotic structure should regenerate HP when not dormant"
        );
    }

    #[test]
    fn test_symbiotic_structure_goes_dormant_if_needs_unmet() {
        let mut app = setup_app();

        // Spawn structure with specific light need
        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                SymbioticStructure {
                    regeneration_rate: 5.0,
                    is_dormant: false,
                },
                SymbioticNeeds {
                    required_light: 100.0,
                    required_temp: 20.0,
                },
                EnvironmentStatus {
                    current_light: 0.0,
                    current_temp: 20.0,
                }, // Light is 0 (eclipse scenario)
            ))
            .id();

        app.update();

        // Structure should become dormant
        let symbiotic = app.world().get::<SymbioticStructure>(entity).unwrap();
        assert!(
            symbiotic.is_dormant,
            "Structure should enter dormancy when light needs are unmet"
        );

        // Dormant structure should NOT regenerate
        app.update();
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(
            structure.current_hp, 50.0,
            "Dormant structure should not regenerate health"
        );
    }
}
