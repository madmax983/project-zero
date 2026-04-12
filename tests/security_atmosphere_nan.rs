#[cfg(test)]
mod tests {
    use bevy_utils::hashbrown::HashMap;
    use scale::layer1::atmosphere::*;
    use scale::layer1::weather::{WeatherState, WeatherType};

    #[test]
    fn test_atmosphere_inf_propagation() {
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(5, 5, f32::INFINITY);

        let blockers = HashMap::new();
        grid.diffuse(&blockers, 0.5);

        let val = grid.get(6, 5);
        assert!(val.is_infinite() || val > 1000.0);
    }

    #[test]
    fn test_update_weather_diffusion_system() {
        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(AtmosphereGrid::new(10, 10));
        world.insert_resource(DiffusionConfig {
            rate: 0.5,
            vertical_escape: 0.1,
        });

        let weather = WeatherState {
            current_weather: WeatherType::ThermalInversion,
            ..Default::default()
        };
        world.insert_resource(weather);

        use bevy_ecs::system::RunSystemOnce;
        world
            .run_system_once(update_weather_diffusion_system)
            .unwrap();

        let grid = world.resource::<AtmosphereGrid>();
        assert_eq!(
            grid.diffusion_rate, 1.0,
            "Thermal Inversion should have 0 escape rate"
        );

        let weather2 = WeatherState {
            current_weather: WeatherType::Clear,
            ..Default::default()
        };
        world.insert_resource(weather2);

        world
            .run_system_once(update_weather_diffusion_system)
            .unwrap();
        let grid2 = world.resource::<AtmosphereGrid>();
        assert_eq!(
            grid2.diffusion_rate, 0.9,
            "Clear weather should use base escape rate"
        );
    }
}
