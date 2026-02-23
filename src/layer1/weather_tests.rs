#[cfg(test)]
mod tests {
    use crate::layer1::atmosphere::{
        AtmosphereGrid, DiffusionConfig, GasType, apply_smog_damage_system,
        simulate_diffusion_system,
    };
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::weather::{WeatherState, WeatherType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_inversion_halts_diffusion() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(20, 20);
        // Set pollution at (10,10)
        grid.set_gas(10, 10, GasType::Smog, 100.0);
        world.insert_resource(grid);

        // Default Config: allows diffusion
        // Note: We set horizontal rate to 0.0 to isolate vertical escape testing,
        // ensuring the drop is only due to escape (or lack thereof).
        // If we followed the spec literally (rate: 0.1), horizontal diffusion would drop the value below 99.0
        // regardless of vertical escape, making the test fail.
        world.insert_resource(DiffusionConfig {
            rate: 0.0,
            vertical_escape: 0.05,
        });

        // Set Weather to Inversion
        world.insert_resource(WeatherState {
            current_weather: WeatherType::ThermalInversion,
            duration_remaining: 100,
        });

        // Run modified diffusion system
        world.run_system_once(simulate_diffusion_system).unwrap();

        let grid = world.get_resource::<AtmosphereGrid>().unwrap();
        let smog = grid.get_gas(10, 10, GasType::Smog);

        // Should be close to 100.0 (no vertical escape)
        // With normal weather, it would lose 5% (to 95.0)
        assert!(smog > 99.0, "Smog level {} should be > 99.0", smog);
    }

    #[test]
    fn test_smog_damage_during_inversion() {
        let mut world = World::new();
        // Setup Pop in smog
        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        // Setup heavy smog at pos
        let mut grid = AtmosphereGrid::new(20, 20);
        grid.set_gas(10, 10, GasType::Smog, 200.0); // Toxic level
        world.insert_resource(grid);

        // Run damage system
        world.run_system_once(apply_smog_damage_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Health should drop from smog");
    }
}
