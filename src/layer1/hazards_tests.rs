#[cfg(test)]
mod tests {
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::execution::{AtTarget, MovementTarget, work_execution_system};
    use crate::layer1::health::Health;
    use crate::layer1::husbandry::{HusbandryConfig, Tame, tame_execution_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, MiningProgress};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_types::ActionType;
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
        crate::setup::init_task_pools();

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

        for _ in 0..5000 {
            // Reset health if damaged to avoid death
            if let Some(mut h) = world.get_mut::<Health>(pop) {
                if h.current < initial_health {
                    took_damage = true;
                    h.current = initial_health; // Reset
                }
            }

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

    #[test]
    fn test_taming_is_dangerous() {
        let mut world = World::new();
        crate::setup::init_task_pools();

        // Resources
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));
        world.insert_resource(HusbandryConfig::default());
        world.insert_resource(MessageLog::default());

        // Create initial designation
        let mut designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Tame,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Create animal
        let animal = world
            .spawn((
                crate::layer1::fauna::Fauna {
                    fauna_type: crate::layer1::fauna::FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Create pop
        let pop = world
            .spawn((
                Pop,
                Health::default(),
                Needs::default(),
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Tame,
                },
                AtTarget,
            ))
            .id();

        let initial_health = world.get::<Health>(pop).unwrap().current;
        let mut took_damage = false;

        for _ in 0..5000 {
            // 1. Heal if needed
            if let Some(mut h) = world.get_mut::<Health>(pop) {
                if h.current < initial_health {
                    took_damage = true;
                    h.current = initial_health;
                }
            }

            // 2. Reset Pop State (MovementTarget removed by system)
            if world.get::<MovementTarget>(pop).is_none() {
                world.entity_mut(pop).insert((
                    MovementTarget {
                        target_entity: designation,
                        target_position: GridPosition { x: 5, y: 5 },
                        for_action: ActionType::Tame,
                    },
                    AtTarget,
                ));
            }

            // 3. Reset Designation (Despawned by system)
            if world.get_entity(designation).is_err() {
                designation = world
                    .spawn((
                        Designation {
                            designation_type: DesignationType::Tame,
                        },
                        GridPosition { x: 5, y: 5 },
                    ))
                    .id();
                // Update target ref
                if let Some(mut mt) = world.get_mut::<MovementTarget>(pop) {
                    mt.target_entity = designation;
                }
            }

            // 4. Reset Animal (Tame component added by system)
            world.entity_mut(animal).remove::<Tame>();

            // Run system
            tame_execution_system(&mut world);

            if took_damage {
                break;
            }
        }

        assert!(took_damage, "Taming should be dangerous");
    }
}
