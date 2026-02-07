#[cfg(test)]
mod tests {
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::execution::{AtTarget, MovementTarget, work_execution_system};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, MiningProgress};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_ai::ActionType;
    use crate::shared::log::MessageLog;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_action_danger_levels() {
        // Mining and Forestry (Work) should be dangerous
        assert!(ActionType::Work.danger_level() > 0.0);

        // Sleeping and Eating should be safe
        assert_eq!(ActionType::SatisfyHunger.danger_level(), 0.0);
        assert_eq!(ActionType::SatisfyRest.danger_level(), 0.0);
    }

    #[test]
    fn test_action_accident_damage() {
        // Work should cause damage
        assert!(ActionType::Work.accident_damage() > 0.0);

        // Others should not
        assert_eq!(ActionType::SatisfyHunger.accident_damage(), 0.0);
    }

    #[test]
    fn test_work_accident_occurs() {
        // Setup world
        let mut world = World::new();
        crate::setup::init_task_pools(); // Initialize task pools for parallel iterators if needed

        // Resources
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // Target
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());

        // Create designation with high difficulty to ensure it lasts long enough
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                MiningProgress {
                    current: 0.0,
                    max: 1_000_000.0, // Should last long enough for accident
                },
            ))
            .id();

        // Create pop
        let pop = world
            .spawn((
                Pop,
                Health::default(), // Important!
                Needs::default(),
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        let initial_health = world.get::<Health>(pop).unwrap().current;
        let mut took_damage = false;

        // Loop to simulate probability
        // With 0.1% chance (0.001), 1000 iterations gives ~63% chance of occurring at least once.
        // We'll increase iterations or verify that probability isn't 0.
        // Or better, we can mock RNG if we refactor, but for now let's try statistical approach with high iterations.
        // Actually, 2000 iterations gives ~86%.

        for _ in 0..5000 {
            // Reset health if damaged to avoid death
            if let Some(mut h) = world.get_mut::<Health>(pop) {
                if h.current < initial_health {
                    took_damage = true;
                    h.current = initial_health; // Reset
                }
            }

            // Run system
            // Note: work_execution_system uses par_iter internally potentially? No, just iter.
            // But it needs &mut World.
            work_execution_system(&mut world);

            if took_damage {
                break;
            }
        }

        assert!(took_damage, "Dangerous work should eventually cause damage");

        // Verify log message
        let log = world.resource::<MessageLog>();
        assert!(
            log.messages.iter().any(|m| m.text.contains("ACCIDENT")),
            "Should log accident"
        );
    }
}
