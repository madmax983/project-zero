use crate::layer1::core::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

pub fn weather_madness_system(
    weather: Res<WeatherState>,
    roof_grid: Res<RoofGrid>,
    mut query: Query<(&mut Needs, &Traits, &GridPosition), With<crate::layer1::pop::Pop>>,
) {
    if weather.current_weather != WeatherType::MutagenicRain
        && weather.current_weather != WeatherType::Storm
    {
        return;
    }

    for (mut needs, traits, pos) in &mut query {
        if traits.has(Trait::Anxious) && !roof_grid.has_roof(pos.x, pos.y) {
            needs.leisure = (needs.leisure - 0.05).max(0.0);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(weather_madness_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_weather_madness_penalizes_exposed_anxious_pops() {
        let mut world = World::new();

        let weather = WeatherState {
            current_weather: WeatherType::Storm,
            ..Default::default()
        };
        world.insert_resource(weather);

        let roof_grid = RoofGrid::new(10, 10);
        world.insert_resource(roof_grid);

        let mut traits = Traits::default();
        traits.add(Trait::Anxious);

        let needs = Needs {
            leisure: 1.0,
            ..Default::default()
        };

        let pop = world
            .spawn((Pop, traits, needs, GridPosition { x: 5, y: 5 }))
            .id();

        world.run_system_once(weather_madness_system).unwrap();

        let needs_after = world.get::<Needs>(pop).unwrap();
        assert!(needs_after.leisure < 1.0);
    }
}
