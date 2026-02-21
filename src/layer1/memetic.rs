//! Memetic Hazards system (Spec 174).
//!
//! Handles the infection of Pops with memetic viruses from hazardous research,
//! and the spread of infection via graffiti.

use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::PopEvalData;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;
use rand::seq::IteratorRandom;

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

/// Evaluates the desire to scrawl memetic sigils on walls.
///
/// This action is available only to Pops infected with [`MemeticCarrier`].
/// It overrides normal priorities with a high utility score (2.0).
pub fn evaluate_scrawl_memetic_sigil(
    data: &PopEvalData,
    world: &mut World,
) -> Option<(ActionType, f32, Option<Entity>)> {
    // 1. Am I a carrier?
    world.get::<MemeticCarrier>(data.entity)?;

    // 2. Find a wall to deface
    let mut rng = rand::thread_rng();

    let mut query = world.query::<(Entity, &GridPosition, &Building)>();
    let candidates: Vec<(Entity, GridPosition)> = query
        .iter(world)
        .filter(|(_, _, b)| b.building_type == BuildingType::Wall)
        .map(|(e, p, _)| (e, *p))
        .collect();

    // Pick one
    if let Some((target, _pos)) = candidates.into_iter().choose(&mut rng) {
        // High utility to override everything else (2.0 vs normal 1.0 max)
        return Some((ActionType::ScrawlMemeticSigil, 2.0, Some(target)));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::graffiti::{
        Graffiti, GraffitiMap, GraffitiType, graffiti_observation_system,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::{Tech, TechState, unlock_tech};
    use crate::layer1::utility_ai::evaluate_actions_system;
    use crate::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

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
        let wall = world
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
