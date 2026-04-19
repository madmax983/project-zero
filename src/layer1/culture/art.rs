//! Cultural Artifacts system (Art).
//!
//! Handles creation of Art on specific buildings (Statues) and observation by Pops.

use crate::layer1::building::{Building, BuildingType};
use crate::layer1::chronicle::Chronicle;
use crate::layer1::map::GridPosition;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::pop::Pop;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component added to buildings that are considered cultural artifacts.
#[derive(Component, Debug, Clone)]
pub struct Art {
    /// Description of what the art depicts.
    pub description: String,
}

/// System to generate Art components on newly built Statues.
pub fn art_generation_system(
    mut commands: Commands,
    query: Query<(Entity, &Building), Added<Building>>,
    chronicle: Res<Chronicle>,
) {
    let mut rng = rand::thread_rng();

    for (entity, building) in &query {
        if building.building_type == BuildingType::Statue {
            let description = if chronicle.events.is_empty() {
                "Abstract Art".to_string()
            } else {
                let idx = rng.gen_range(0..chronicle.events.len());
                let event = &chronicle.events[idx];
                format!("Art depicting: {}", event.text)
            };

            commands.entity(entity).insert(Art { description });
        }
    }
}

/// System to allow pops to observe nearby Art and gain mood buffs.
pub fn art_observation_system(
    art_query: Query<(&GridPosition, &Art)>,
    mut pop_query: Query<(&GridPosition, &mut Memories), With<Pop>>,
    time: Res<SimulationTime>,
) {
    // Brute force O(N*M) is acceptable for MVP as number of statues is low.
    // Optimization: Use spatial index if performance becomes an issue.
    for (pop_pos, mut memories) in &mut pop_query {
        for (art_pos, _) in &art_query {
            // Chebyshev distance
            if pop_pos.distance_chebyshev(*art_pos) <= 2 {
                // Check if already has memory to avoid spamming
                let has_memory = memories
                    .items
                    .iter()
                    .any(|m| m.memory_type == MemoryType::AdmiredArt);

                if !has_memory {
                    memories.add(MemoryType::AdmiredArt, time.tick);
                    // Only admire one piece per tick
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::chronicle::{Chronicle, EventImportance};
    use crate::layer1::map::GridPosition;
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;

    // 1. Art Creation
    #[test]
    fn test_statue_gains_art_component() {
        let mut world = World::new();
        // Setup dependencies
        world.insert_resource(Chronicle::default());
        // Initialize Art plugin/systems if needed, or run manually

        // Spawn a Statue
        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system that assigns art
        // (Assuming `art_generation_system` runs on Added<Building>)
        // Use Bevy's system runner
        let mut schedule = Schedule::default();
        schedule.add_systems(art_generation_system);
        schedule.run(&mut world);

        // Assert Art component exists
        let art = world.get::<Art>(statue);
        assert!(art.is_some(), "Statue should have Art component");
    }

    #[test]
    fn test_art_captures_chronicle_event() {
        let mut world = World::new();
        let mut chronicle = Chronicle::default();
        chronicle.add_event(0, "Colony Founded".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(art_generation_system);
        schedule.run(&mut world);

        let art = world.get::<Art>(statue).expect("Component should exist or System should run");
        assert!(!art.description.is_empty());
        assert!(art.description.contains("Colony Founded"));
    }

    #[test]
    fn test_art_fallback_if_chronicle_empty() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default()); // Empty

        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(art_generation_system);
        schedule.run(&mut world);

        let art = world.get::<Art>(statue).expect("Component should exist or System should run");
        assert_eq!(art.description, "Abstract Art");
    }

    // 2. Art Observation
    #[test]
    fn test_observing_art_gives_memory_buff() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        // Spawn Art
        world.spawn((
            Building {
                building_type: BuildingType::Statue,
            },
            GridPosition { x: 5, y: 5 },
            Art {
                description: "Great Art".to_string(),
            },
        ));

        // Spawn Pop nearby
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 }, // Same tile
                Memories::default(),
            ))
            .id();

        // Run observation system
        let mut schedule = Schedule::default();
        schedule.add_systems(art_observation_system);
        schedule.run(&mut world);

        // Check Pop memories
        let memories = world.get::<Memories>(pop).expect("Component should exist or System should run");
        // Assume we add a specific "AdmiredArt" memory type
        let found = memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::AdmiredArt);
        assert!(found, "Pop should have AdmiredArt memory");
    }

    #[test]
    fn test_observing_art_distance_limit() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        // Spawn Art
        world.spawn((
            Building {
                building_type: BuildingType::Statue,
            },
            GridPosition { x: 5, y: 5 },
            Art {
                description: "Great Art".to_string(),
            },
        ));

        // Spawn Pop far away
        let pop = world
            .spawn((Pop, GridPosition { x: 10, y: 10 }, Memories::default()))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(art_observation_system);
        schedule.run(&mut world);

        let memories = world.get::<Memories>(pop).expect("Component should exist or System should run");
        let found = memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::AdmiredArt);
        assert!(!found, "Pop too far away should not admire art");
    }
}
