//! Memetic Hazards system (Spec 174).
//!
//! Handles the infection of Pops with memetic viruses from hazardous research,
//! and the spread of infection via graffiti.

use bevy_ecs::prelude::*;

/// Component marking a Pop as a carrier of a memetic virus.
///
/// Carriers prioritize spreading the virus (e.g., scrawling sigils) over normal needs/work.
#[derive(Component, Default)]
pub struct MemeticCarrier;

/// Configuration for Memetic Hazards.
#[derive(Resource, Debug, Clone)]
pub struct MemeticConfig {
    /// Chance (0.0 - 1.0) for a pop to become infected when observing a Memetic Sigil.
    pub infection_chance: f64,
}

impl Default for MemeticConfig {
    fn default() -> Self {
        Self {
            infection_chance: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::graffiti::{Graffiti, GraffitiMap, GraffitiType, graffiti_observation_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::tech::{Tech, TechState, unlock_tech};
    use crate::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};
    use crate::layer1::utility_ai::evaluate_actions_system;
    use crate::layer1::resources::ColonyResources;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_unlocking_hazardous_tech_infects_researcher() {
        let mut world = World::new();
        // Setup TechState, Resources, and a Researcher Pop
        let mut tech_state = TechState::default();
        tech_state.total_capacity = 1000.0; // Ensure enough capacity
        world.insert_resource(tech_state);
        world.insert_resource(ColonyResources {
            knowledge: 1000.0, // Plenty of knowledge
            ..Default::default()
        });
        world.insert_resource(crate::shared::log::MessageLog::default());

        // We need to loop this test or ensure infection happens.
        // Infection is random selection from candidates.
        // If there is only 1 candidate, it should be selected 100%?
        // Let's check tech.rs logic.
        // `candidates.into_iter().choose(&mut rng)`
        // `choose` from 1 candidate is 100%.

        let researcher = world.spawn((
            Pop,
            AssignedTo { assignment_type: AssignmentType::LibraryWorker, entity: Entity::PLACEHOLDER }, // entity placeholder is fine for assignment type check
            GridPosition::default()
        )).id();

        // Define a hazardous tech (e.g. VoidWhispers)
        // Unlock it
        // Note: Tech::VoidWhispers doesn't exist yet, this will fail compilation (RED phase)
        let result = unlock_tech(&mut world, Tech::VoidWhispers);

        // Assert success
        assert!(result);
        // Assert researcher is infected
        assert!(world.get::<MemeticCarrier>(researcher).is_some());
    }

    #[test]
    fn test_infected_pop_scrawls_sigil() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        // Setup infected pop
        let pop = world.spawn((
            Pop,
            MemeticCarrier,
            GridPosition { x: 5, y: 5 },
            UtilityWeights::default(),
            PopAction {
                ticks_committed: 100, // Ensure ready for evaluation
                ..Default::default()
            },
            Needs::default(),
        )).id();

        // Setup wall at (5,6) to scrawl on
        let _wall = world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 5, y: 6 },
            // OccupiedTiles is usually a resource, but utility AI might use it for validity checks?
            // Actually, evaluate_scrawl_memetic_sigil will likely query Buildings directly or use OccupiedTiles.
            // Let's ensure OccupiedTiles exists and has the wall.
        )).id();

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 6));
        world.insert_resource(occupied);

        // Insert necessary resources (UtilityConfig, etc) done above.

        // Evaluate actions
        evaluate_actions_system(&mut world);

        // Assert Pop chose ScrawlMemeticSigil
        let action = world.get::<PopAction>(pop).unwrap();
        // ActionType::ScrawlMemeticSigil doesn't exist yet
        assert_eq!(action.current, ActionType::ScrawlMemeticSigil);
    }

    #[test]
    fn test_observing_sigil_spreads_infection() {
        let mut world = World::new();
        // Setup clean pop
        let victim = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Morale::default()
        )).id();

        // Place Memetic Sigil at (5,6)
        let mut map = GraffitiMap::default();
        map.markings.insert(
            (5, 6),
            Graffiti {
                // GraffitiType::MemeticSigil doesn't exist yet
                graffiti_type: GraffitiType::MemeticSigil,
                decay: 100.0,
                modifier: -0.1
            }
        );
        world.insert_resource(map);

        // Run observation system
        // Set high infection chance for test
        world.insert_resource(MemeticConfig { infection_chance: 1.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_observation_system);
        schedule.run(&mut world);

        // Assert victim is infected
        assert!(world.get::<MemeticCarrier>(victim).is_some());
    }
}
