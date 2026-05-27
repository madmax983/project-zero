#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::nature::weather::{WeatherState, WeatherType};
    use crate::layer1::nature::weather_fronts::{WeatherFront, WeatherFronts, update_weather_fronts_system};
    use crate::shared::time::SimulationTime;
    use crate::layer1::core::chronicle::Chronicle;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<WeatherFronts>();
        world.init_resource::<WeatherState>();
        world.init_resource::<SimulationTime>();
        world.init_resource::<Chronicle>();
        world
    }

    #[test]
    fn test_front_movement() {
        let mut world = setup_world();
        let front = WeatherFront {
            weather_type: WeatherType::Storm,
            distance_km: 1000.0,
            speed_km_h: 100.0,
            duration_h: 24.0,
        };

        world.resource_mut::<WeatherFronts>().active_fronts.push(front);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_weather_fronts_system);
        schedule.run(&mut world);

        let fronts = world.resource::<WeatherFronts>();
        assert!(fronts.active_fronts[0].distance_km < 1000.0, "Front should move closer");
    }

    #[test]
    fn test_front_arrival_overrides_weather() {
        let mut world = setup_world();

        let front = WeatherFront {
            weather_type: WeatherType::Storm,
            distance_km: 0.0,
            speed_km_h: 100.0,
            duration_h: 10.0,
        };

        world.resource_mut::<WeatherFronts>().active_fronts.push(front);
        world.resource_mut::<WeatherState>().current_weather = WeatherType::Clear;

        let mut schedule = Schedule::default();
        schedule.add_systems(update_weather_fronts_system);
        schedule.run(&mut world);

        let weather = world.resource::<WeatherState>();
        assert_eq!(weather.current_weather, WeatherType::Storm, "Front should override weather");
        assert!(weather.duration_remaining > 0);
    }

    #[test]
    fn test_front_passing() {
        let mut world = setup_world();

        let front = WeatherFront {
            weather_type: WeatherType::Storm,
            distance_km: 0.0,
            speed_km_h: 100.0,
            duration_h: 0.0,
        };

        world.resource_mut::<WeatherFronts>().active_fronts.push(front);
        world.resource_mut::<WeatherState>().current_weather = WeatherType::Storm;

        let mut schedule = Schedule::default();
        schedule.add_systems(update_weather_fronts_system);
        schedule.run(&mut world);

        let fronts = world.resource::<WeatherFronts>();
        assert!(fronts.active_fronts.is_empty(), "Expired front should be removed");

        let chronicle = world.resource::<Chronicle>();
        assert!(chronicle.events.iter().any(|e| e.text.contains("passed")));
    }
}
