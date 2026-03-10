#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::geodetic::{form_golem_system, update_living_stone_system, LivingStone, StoneGolem};
    use scale::layer1::items::{Item, ItemType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::temperature::TemperatureGrid;
    use scale::shared::time::SimulationTime;

    fn setup_world() -> (World, Schedule) {
        scale::setup::init_task_pools();
        let mut world = World::new();

        // Initialize only what we need for the isolated systems.
        world.insert_resource(SimulationTime {
            tick: 1000,
            ..Default::default()
        });

        // Add schedule with isolated systems
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                update_living_stone_system,
                form_golem_system.after(update_living_stone_system)
            )
        );

        (world, schedule)
    }

    #[test]
    fn test_geodetic_migration_heat() {
        let (mut world, mut schedule) = setup_world();

        // Setup heat source at (5, 5)
        let mut temp_grid = TemperatureGrid::new(10, 10, 20.0);
        temp_grid.set(5, 5, 100.0);
        world.insert_resource(temp_grid);

        // Spawn a LivingStone at (1, 1)
        let id = world
            .spawn((
                Item {
                    item_type: ItemType::LivingStone,
                    ..Default::default()
                },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 1, y: 1 },
            ))
            .id();

        // Run simulation for enough ticks
        // MOVE_INTERVAL is 100 ticks, we run 5 times
        for _ in 0..505 {
            let mut time = world.resource_mut::<SimulationTime>();
            time.tick += 1;
            schedule.run(&mut world);
        }

        let pos = world.get::<GridPosition>(id).unwrap();

        // Should have moved towards (5, 5)
        assert!(
            pos.x > 1 || pos.y > 1,
            "Stone should move towards heat. Pos: {:?}",
            pos
        );
    }

    #[test]
    fn test_geodetic_golem_formation() {
        let (mut world, mut schedule) = setup_world();
        world.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        world.insert_resource(Option::<scale::shared::log::MessageLog>::None); // Optional resource handled gracefully

        // Spawn 5 LivingStones at (5, 5)
        for _ in 0..5 {
            world.spawn((
                Item {
                    item_type: ItemType::LivingStone,
                    ..Default::default()
                },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 5, y: 5 },
            ));
        }

        schedule.run(&mut world);

        let golem_count = world.query::<&StoneGolem>().iter(&world).count();
        assert_eq!(golem_count, 1, "Should form one Golem");

        let stone_count = world.query::<&LivingStone>().iter(&world).count();
        assert_eq!(stone_count, 0, "Stones should be consumed");
    }
}
