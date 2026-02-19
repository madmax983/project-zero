#[cfg(test)]
mod integration_tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::funeral::{Corpse, Grave};
    use scale::layer1::health::Health;
    use scale::layer1::map::GridPosition;
    use scale::layer1::memory::{Memories, MemoryType};
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::{Pop, PopName};
    use scale::layer1::utility_types::{PopAction, UtilityConfig, UtilityWeights};
    use scale::shared::time::SimulationTime;
    use scale::simulation::run_simulation_tick;

    fn setup_world() -> World {
        scale::setup::setup_world()
    }

    #[test]
    fn test_death_spawns_corpse() {
        let mut world = setup_world();
        world.insert_resource(scale::shared::state::GameState::Running);
        world.insert_resource(SimulationTime::default());

        let entity = world
            .spawn((
                Pop,
                PopName("TestSubject".to_string()),
                Health {
                    current: -10.0,
                    max: 100.0,
                }, // Dead
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run simulation tick (which includes death_system)
        run_simulation_tick(&mut world);

        // Pop should be despawned
        assert!(
            world.get_entity(entity).is_err(),
            "Pop entity should be despawned"
        );

        // Corpse should be spawned at same location
        let mut query = world.query::<(&Corpse, &GridPosition)>();
        let corpse = query.single(&world);
        assert_eq!(corpse.0.name, "TestSubject");
        assert_eq!(corpse.1.x, 5);
        assert_eq!(corpse.1.y, 5);
    }

    #[test]
    fn test_funeral_integration_death_to_burial() {
        let mut world = setup_world();
        world.insert_resource(scale::shared::state::GameState::Running);
        world.insert_resource(SimulationTime::default());
        world.insert_resource(UtilityConfig {
            evaluation_interval: 1, // Evaluate every tick
            ..Default::default()
        });

        // Despawn initial pops to avoid interference
        let initial_pops: Vec<Entity> = world
            .query_filtered::<Entity, With<Pop>>()
            .iter(&world)
            .collect();
        for e in initial_pops {
            world.despawn(e);
        }

        // 1. Create a corpse (simulate death having happened)
        let corpse_entity = world
            .spawn((
                Corpse {
                    name: "Deceased".to_string(),
                    decay: 0.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // 2. Create a grave
        let grave_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Grave,
                },
                GridPosition { x: 8, y: 5 }, // Close by
                Grave {
                    occupied: false,
                    corpse_name: None,
                },
            ))
            .id();

        // 3. Create an undertaker pop
        let undertaker = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 5 },
                Needs::default(), // Not hungry/tired
                PopAction::default(),
                UtilityWeights::default(),
                Memories::default(),
            ))
            .id();

        // Run simulation ticks
        // We run multiple ticks to allow travel and work
        for _ in 0..300 {
            run_simulation_tick(&mut world);
        }

        // Verify Corpse is gone (buried)
        assert!(
            world.get_entity(corpse_entity).is_err(),
            "Corpse should be buried (despawned)"
        );

        // Verify Grave is occupied
        let grave = world.get::<Grave>(grave_entity).unwrap();
        assert!(grave.occupied, "Grave should be occupied");
        assert_eq!(grave.corpse_name.as_deref(), Some("Deceased"));

        // Verify Closure memory on undertaker
        let memories = world.get::<Memories>(undertaker).unwrap();
        assert!(
            memories
                .items
                .iter()
                .any(|m| m.memory_type == MemoryType::AttendedFuneral),
            "Undertaker should have AttendedFuneral memory"
        );
    }

    #[test]
    fn test_grief_from_corpse() {
        let mut world = setup_world();
        world.insert_resource(scale::shared::state::GameState::Running);
        world.insert_resource(SimulationTime::default());

        // Spawn Corpse
        world.spawn((
            Corpse {
                name: "Unfortunate Soul".to_string(),
                decay: 0.5,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Witness nearby
        let witness = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 }, // Adjacent
                Needs::default(),
                Memories::default(),
            ))
            .id();

        // Run simulation tick (grief_system runs)
        run_simulation_tick(&mut world);

        // Check for SawCorpse memory
        let memories = world.get::<Memories>(witness).unwrap();
        assert!(
            memories
                .items
                .iter()
                .any(|m| m.memory_type == MemoryType::SawCorpse),
            "Witness should have SawCorpse memory"
        );
    }
}
