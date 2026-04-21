//! Energy fire mechanics tests
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::energy::{power_grid_system, GridOverloadEvent, PowerConsumer, PowerSource};
    use scale::layer1::fire::Fire;
    use scale::layer1::health::Health;
    use scale::layer1::integration::grid_overload_fire_bridge;
    use scale::layer1::map::GridPosition;

    #[test]
    fn test_overload_starts_fire() {
        let mut world = World::new();

        // Initialize events
        world.init_resource::<Events<GridOverloadEvent>>();

        // Generator at 0,0
        world.spawn((
            PowerSource {
                output: 10.0,
                active: true,
            },
            GridPosition { x: 0, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
                conditions: Vec::new(),
            },
        ));

        // Consumer at 0,1 (Connected via adjacency)
        // Demand 30 vs Output 10 => Ratio 3.0 (> 1.5 threshold)
        world.spawn((
            PowerConsumer {
                demand: 30.0,
                active: true,
            },
            GridPosition { x: 0, y: 1 },
            Health {
                current: 100.0,
                max: 100.0,
                conditions: Vec::new(),
            },
        ));

        // Create a schedule to run systems
        let mut schedule = Schedule::default();
        schedule.add_systems((
            power_grid_system,
            grid_overload_fire_bridge.after(power_grid_system),
        ));

        // Run system enough times to trigger probability
        let mut fire_spawned = false;
        for _ in 0..200 {
            // Clear old events
            world.resource_mut::<Events<GridOverloadEvent>>().update();

            schedule.run(&mut world);

            // Check for Fire entity
            let fire_count = world.query::<&Fire>().iter(&world).count();
            if fire_count > 0 {
                fire_spawned = true;
                break;
            }
        }

        assert!(
            fire_spawned,
            "Severe overload should eventually start a fire"
        );
    }
}
