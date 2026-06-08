//! Chemical Showers (Nova Feature)
//!
//! # The Spark
//! I noticed Pops use showers to clean off filth, but we also have a fully fledged
//! chemical/addiction system. What if the colony administration spiked the water supply?
//!
//! # The Feature
//! Adds a `ChemicalShower` component that can be attached to a Shower building.
//! When a Pop is executing `ActionType::UseShower` at that building, they ingest
//! the chemical, applying its effects (e.g. Stim, Sedative) globally to their state.
//!
//! # The Potential
//! Players can deliberately drug their population via the hygiene system to
//! enforce productivity (Stims) or suppress civil unrest (Sedatives) en masse.

use crate::layer1::chemical::{consume_chemical, ChemicalType};
use crate::layer1::execution::components::{AtTarget, MovementTarget};
    use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

/// A component that turns a standard shower into a chemical delivery vector.
#[derive(Component, Debug, Clone)]
pub struct ChemicalShower {
    /// The chemical injected into the water supply.
    pub chemical: ChemicalType,
}

/// System that applies chemicals to pops actively using a ChemicalShower.
pub fn chemical_showers_system(world: &mut World) {
    // 1. Identify which pops are currently using a shower at their target
    let mut targets_to_medicate = Vec::new();

    // Query for pops that are executing the UseShower action and are at their target building.
    let mut q = world.query_filtered::<(Entity, &MovementTarget), With<AtTarget>>();

    for (pop_entity, mt) in q.iter(world) {
        if mt.for_action == ActionType::UseShower {
            targets_to_medicate.push((pop_entity, mt.target_entity));
        }
    }

    // 2. Resolve the target buildings to see if they have ChemicalShower
    // We do this in two steps to avoid holding a borrow on the World while mutating it
    let mut chemical_applications = Vec::new();

    for (pop_entity, shower_entity) in targets_to_medicate {
        // Fetch the ChemicalShower component if it exists
        if let Some(chem_shower) = world.get::<ChemicalShower>(shower_entity) {
            chemical_applications.push((pop_entity, chem_shower.chemical));
        }
    }

    // 3. Apply the chemicals using the existing `consume_chemical` pipeline
    for (pop_entity, chem_type) in chemical_applications {
        consume_chemical(world, pop_entity, chem_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chemical::{ChemicalState, ChemicalType};
    use crate::layer1::architecture::Building;
    use crate::layer1::map::GridPosition;
    use crate::layer1::execution::components::{AtTarget, MovementTarget};
        use crate::layer1::utility_types::ActionType;

    #[test]
    fn test_chemical_showers_applies_chemical() {
        let mut world = World::new();

        // Setup essential resources for consume_chemical
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Spawn a spiked shower
        let shower = world
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::BuildingType::Shower,
                },
                ChemicalShower {
                    chemical: ChemicalType::Stim,
                },
            ))
            .id();

        // Spawn a pop taking a shower
        let pop = world
            .spawn((
                crate::layer1::pop::Pop,
                MovementTarget {
                    target_entity: shower,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::UseShower,
                },
                AtTarget,
                ChemicalState::default(),
            ))
            .id();

        // Run the system
        chemical_showers_system(&mut world);

        // Verify the pop received the chemical
        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(
            !state.active_effects.is_empty(),
            "Pop should have active effects from the shower"
        );
        assert_eq!(state.active_effects[0].chemical, ChemicalType::Stim);
        assert_eq!(
            state.active_effects[0].magnitude, 1.5,
            "Stim magnitude should be applied"
        );
    }

    #[test]
    fn test_normal_showers_do_not_apply_chemical() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Spawn a normal shower (no ChemicalShower component)
        let shower = world
            .spawn(Building {
                building_type: crate::layer1::architecture::BuildingType::Shower,
            })
            .id();

        // Spawn a pop taking a shower
        let pop = world
            .spawn((
                crate::layer1::pop::Pop,
                MovementTarget {
                    target_entity: shower,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::UseShower,
                },
                AtTarget,
                ChemicalState::default(),
            ))
            .id();

        chemical_showers_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(
            state.active_effects.is_empty(),
            "Pop should NOT have active effects from a normal shower"
        );
    }

    #[test]
    fn test_chemical_showers_only_applies_when_using_shower() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Spawn a spiked shower
        let shower = world
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::BuildingType::Shower,
                },
                ChemicalShower {
                    chemical: ChemicalType::Sedative,
                },
            ))
            .id();

        // Spawn a pop doing something else (e.g. Repair) at the shower
        let pop = world
            .spawn((
                crate::layer1::pop::Pop,
                MovementTarget {
                    target_entity: shower,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Repair, // Not showering!
                },
                AtTarget,
                ChemicalState::default(),
            ))
            .id();

        chemical_showers_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(
            state.active_effects.is_empty(),
            "Pop should NOT get sedated while repairing the shower"
        );
    }
}
