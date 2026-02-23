#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerSource, update_auroral_output_system};
    use crate::layer1::weather::{WeatherState, WeatherType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::lighting::LightSource;

    #[test]
    fn test_auroral_collector_variant_exists() {
        // Ensure the building type exists
        let collector = BuildingType::AuroralCollector;
        assert_eq!(collector.label(), "Auroral Collector");
    }

    #[test]
    fn test_auroral_output_zero_in_clear_weather() {
        let mut world = World::new();

        // Setup Weather: Clear
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        // Spawn Collector
        let collector = world.spawn((
            PowerSource { output: 10.0, active: true }, // Initial dummy value
            Building { building_type: BuildingType::AuroralCollector },
            LightSource::default(), // For visual feedback check
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_auroral_output_system);
        schedule.run(&mut world);

        // Verify Output is 0.0
        let source = world.get::<PowerSource>(collector).unwrap();
        assert_eq!(source.output, 0.0, "Output should be 0.0 in Clear weather");

        // Verify Light is off
        let light = world.get::<LightSource>(collector).unwrap();
        assert_eq!(light.intensity, 0.0, "Light intensity should be 0.0 in Clear weather");
    }

    #[test]
    fn test_auroral_output_high_in_magnetic_storm() {
        let mut world = World::new();

        // Setup Weather: Magnetic Storm
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        // Spawn Collector
        let collector = world.spawn((
            PowerSource { output: 0.0, active: true },
            Building { building_type: BuildingType::AuroralCollector },
            LightSource::default(),
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_auroral_output_system);
        schedule.run(&mut world);

        // Verify Output is High (e.g., 50.0)
        let source = world.get::<PowerSource>(collector).unwrap();
        assert_eq!(source.output, 50.0, "Output should be 50.0 during Magnetic Storm");

        // Verify Light is on
        let light = world.get::<LightSource>(collector).unwrap();
        assert!(light.intensity > 0.0, "Light intensity should be active during storm");
    }

    #[test]
    fn test_other_buildings_ignored() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        // Normal Generator
        let generator = world.spawn((
            PowerSource { output: 10.0, active: true },
            Building { building_type: BuildingType::Generator },
            crate::layer1::lighting::LightSource::default(),
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_auroral_output_system);
        schedule.run(&mut world);

        // Output should remain unchanged by *this* system
        let source = world.get::<PowerSource>(generator).unwrap();
        assert_eq!(source.output, 10.0, "Normal Generator should not be modified by Auroral system");
    }
}
