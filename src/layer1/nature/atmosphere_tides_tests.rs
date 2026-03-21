#[cfg(test)]
mod tests {
    use crate::layer1::atmosphere::{
        sync_global_wind_system, update_atmospheric_tide_system, AtmosphericTide, BaseGlobalWind,
    };
    use crate::layer1::erosion::ErosionGrid;
    use crate::layer1::execution::{movement_system, MovementTarget};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_types::ActionType;
    use crate::layer1::wind::GlobalWind;
    use crate::layer1::wind::Vec2; // Using local Vec2 from wind.rs
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_tide_oscillation() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(AtmosphericTide::default());

        // Default pressure should be 1.0
        assert!((world.resource::<AtmosphericTide>().pressure - 1.0).abs() < f32::EPSILON);

        // Advance time to 1/4 cycle (Peak High Pressure)
        // Assuming cycle is 1000 ticks
        let mut time = SimulationTime::default();
        time.tick = 250;
        world.insert_resource(time);

        // Update tide
        let mut schedule = Schedule::default();
        schedule.add_systems(update_atmospheric_tide_system);
        schedule.run(&mut world);

        let pressure = world.resource::<AtmosphericTide>().pressure;
        assert!(pressure > 1.2, "Pressure should be high (got {})", pressure);

        // Advance to 3/4 cycle (Peak Low Pressure)
        let mut time = SimulationTime::default();
        time.tick = 750;
        world.insert_resource(time);
        schedule.run(&mut world);

        let pressure = world.resource::<AtmosphericTide>().pressure;
        assert!(pressure < 0.8, "Pressure should be low (got {})", pressure);
    }

    #[test]
    fn test_wind_sync() {
        let mut world = World::new();
        world.insert_resource(BaseGlobalWind {
            speed: 10.0,
            direction: Vec2::X,
        });
        world.insert_resource(GlobalWind::default());

        // High Pressure
        world.insert_resource(AtmosphericTide { pressure: 1.5 });

        let mut schedule = Schedule::default();
        schedule.add_systems(sync_global_wind_system);
        schedule.run(&mut world);

        let effective = world.resource::<GlobalWind>();
        // Speed should be Base * Pressure
        assert!(
            (effective.speed - 15.0).abs() < 0.01,
            "Wind speed should scale with pressure"
        );
        assert_eq!(effective.direction, Vec2::X, "Direction should persist");
    }

    #[test]
    fn test_movement_cost_integration() {
        // This test requires mocking or integrating with movement_system.
        // For unit testing, we can expose a cost calculation function.
        use crate::layer1::atmosphere::calculate_atmospheric_movement_cost;

        // High Pressure (Thick Air) -> Harder to move
        let cost = calculate_atmospheric_movement_cost(1.5);
        assert!(cost > 1.0);

        // Low Pressure (Thin Air) -> Easier to move (less resistance)
        let cost = calculate_atmospheric_movement_cost(0.5);
        assert!(cost < 1.0);
    }

    #[test]
    fn test_movement_system_with_tides() {
        let mut world = World::new();
        // Setup Grid
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(ErosionGrid::new(10, 10));
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Setup Wind (No wind for this test to isolate pressure)
        world.insert_resource(crate::layer1::wind::WindGrid::new(10, 10));

        // Setup High Pressure (1.5) -> Cost 1.2
        world.insert_resource(AtmosphericTide { pressure: 1.5 });

        // Spawn Pop with Speed accumulator 1.0
        // Cost is 1.0 (Grass) * 1.0 (No Wind) * 1.2 (Pressure) = 1.2
        // With accumulator 1.0, they should NOT move (1.0 < 1.2)
        // Even with Coyote Threshold (0.2), 1.0 < (1.2 - 0.2) is false (1.0 == 1.0)?
        // Wait, coyote is: if acc >= cost - 0.2
        // 1.0 >= 1.2 - 0.2 = 1.0. True. They WOULD move.
        // Let's set accumulator to 0.9.
        // 0.9 >= 1.0 (cost-coyote) -> False. No move.
        // If pressure was 1.0 (Normal), cost 1.0. 0.9 >= 0.8. True. Move.

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
                Speed {
                    base: 1.0,
                    current: 0.0, // Don't add speed
                    accumulator: 0.84,
                },
            ))
            .id();

        // Run system
        // We need to use RunSystemOnce because movement_system has many params
        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, movement_system).unwrap();

        // Check position
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(
            pos.x, 0,
            "High pressure should prevent movement at 0.84 accumulator"
        );

        // Now lower pressure to 0.5 -> Cost 0.8
        // Coyote threshold: 0.8 - 0.2 = 0.6.
        // Accumulator 0.9 >= 0.6. Should move.
        world.insert_resource(AtmosphericTide { pressure: 0.5 });

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, movement_system).unwrap();

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(
            pos.x, 1,
            "Low pressure should allow movement at 0.84 accumulator"
        );
    }
}
