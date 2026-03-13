#![allow(clippy::must_use_candidate)]
//! Memetic Hazards System (Spec 174).
//!
//! This module implements "Memetic Hazards", viral ideas that infect Pops and compel them to
//! spread the infection via graffiti (Memetic Sigils).
//!
//! # The Infection Cycle
//!
//! 1.  **Patient Zero**: A researcher unlocks a hazardous technology (e.g., `VoidWhispers`).
//!     This triggers an immediate infection (adds [`MemeticCarrier`] component).
//! 2.  **Compulsion**: Infected Pops have a high-priority action: [`evaluate_scrawl_memetic_sigil`].
//!     They will ignore work and needs to draw Sigils on walls.
//! 3.  **Transmission**: Clean Pops who observe these Sigils (via [`crate::layer1::graffiti::graffiti_observation_system`])
//!     have a chance to become infected (defined in [`MemeticConfig`]).
//! 4.  **Epidemic**: If unchecked, the colony descends into madness as everyone stops working to draw symbols.
//!
//! # Counterplay
//!
//! *   **Cleaning**: Janitors can clean graffiti (removing the vector).
//! *   **Quarantine**: Restricting access to infected areas.
//! *   **Therapy**: Medical treatment (Future Spec).

use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;

/// Component marking a Pop as a carrier of a memetic virus.
///
/// Carriers prioritize spreading the virus (e.g., scrawling sigils) over normal needs/work.
/// This condition is persistent until cured (if a cure exists).
#[derive(Component, Default, Debug)]
pub struct MemeticCarrier;

/// Configuration for Memetic Hazards.
#[derive(Resource, Debug, Clone)]
pub struct MemeticConfig {
    /// Chance (0.0 - 1.0) for a pop to become infected when observing a Memetic Sigil.
    ///
    /// Default: 0.1 (10%).
    pub infection_chance: f64,
}

impl Default for MemeticConfig {
    fn default() -> Self {
        Self {
            infection_chance: 0.1,
        }
    }
}

/// Evaluates the desire to scrawl memetic sigils on walls.
///
/// # Behavior
/// *   **Condition**: Only runs if the Pop has the [`MemeticCarrier`] component.
/// *   **Target**: Selects a random wall from the `UtilityAIBuffer`.
/// *   **Priority**: Returns a utility of **2.0**, which overrides almost all other actions
///     (normal max utility is 1.0). This represents an irresistible compulsion.
pub fn evaluate_scrawl_memetic_sigil(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    // 1. Am I a carrier?
    if !data.is_memetic_carrier {
        return None;
    }

    // 2. Find a wall to deface
    let mut rng = rand::thread_rng();

    // Pick one
    if let Some(target) = buffer.walls.choose(&mut rng) {
        // High utility to override everything else (2.0 vs normal 1.0 max)
        return Some((ActionType::ScrawlMemeticSigil, 2.0, Some(target.entity)));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::graffiti::{
        graffiti_observation_system, Graffiti, GraffitiMap, GraffitiType,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::{unlock_tech, Tech, TechState};
    use crate::layer1::utility_ai::evaluate_actions_system;
    use crate::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};
    use crate::shared::time::SimulationTime;
    // use bevy_ecs::prelude::*; // Already imported via super

    #[test]
    fn test_unlocking_hazardous_tech_infects_researcher() {
        let mut world = World::new();
        // Setup TechState, Resources, and a Researcher Pop
        let tech_state = TechState {
            total_capacity: 1000.0,
            ..Default::default()
        };
        // tech_state.total_capacity = 1000.0; // Ensure enough capacity
        world.insert_resource(tech_state);
        world.insert_resource(ColonyResources {
            knowledge: 1000.0, // Plenty of knowledge
            ..Default::default()
        });
        world.insert_resource(crate::shared::log::MessageLog::default());

        let researcher = world
            .spawn((
                Pop,
                AssignedTo {
                    assignment_type: AssignmentType::LibraryWorker,
                    entity: Entity::PLACEHOLDER,
                }, // entity placeholder is fine for assignment type check
                GridPosition::default(),
            ))
            .id();

        // Unlock Tech::VoidWhispers
        let result = unlock_tech(&mut world, Tech::VoidWhispers);

        // Assert success
        assert!(result);
        // Assert researcher is infected
        assert!(world.get::<MemeticCarrier>(researcher).is_some());
    }

    #[test]
    fn test_infected_pop_scrawls_sigil() {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        // Setup infected pop
        let pop = world
            .spawn((
                Pop,
                MemeticCarrier,
                GridPosition { x: 5, y: 5 },
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 100, // Ensure ready for evaluation
                    ..Default::default()
                },
                Needs::default(),
            ))
            .id();

        // Setup wall at (5,6) to scrawl on
        let _wall = world
            .spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x: 5, y: 6 },
            ))
            .id();

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 6));
        world.insert_resource(occupied);

        // Evaluate actions
        evaluate_actions_system(&mut world);

        // Assert Pop chose ScrawlMemeticSigil
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::ScrawlMemeticSigil);
    }

    #[test]
    fn test_observing_sigil_spreads_infection() {
        let mut world = World::new();
        // Setup clean pop
        let victim = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Morale::default()))
            .id();

        // Place Memetic Sigil at (5,6)
        let mut map = GraffitiMap::default();
        map.markings.insert(
            (5, 6),
            Graffiti {
                graffiti_type: GraffitiType::MemeticSigil,
                decay: 100.0,
                modifier: -0.1,
            },
        );
        world.insert_resource(map);

        // Run observation system
        // Set high infection chance for test
        world.insert_resource(MemeticConfig {
            infection_chance: 1.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_observation_system);
        schedule.run(&mut world);

        // Assert victim is infected
        assert!(world.get::<MemeticCarrier>(victim).is_some());
    }
}
