#![allow(clippy::type_complexity)]
//! Weather Madness (Nova Feature).
//!
//! # The Spark
//! We have various harsh weather types (`WeatherType::Storm`, `MutagenicRain`) and Pops have `StressTracker`s. What if extreme weather could actively drive exposed Pops mad?
//!
//! # The Feature
//! A system `weather_madness_system` that checks if the current weather is a dangerous type (e.g. `MagneticStorm` or `ThermalInversion`) and, for any `Pop` outside (not under a `RoofGrid`), it rapidly increases their stress.

use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::stress::StressTracker;
use bevy_ecs::prelude::*;

const WEATHER_STRESS_PENALTY: f32 = 0.5;

pub fn weather_madness_system(
    weather_state: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut pops: Query<(&GridPosition, &mut StressTracker), With<Pop>>,
) {
    if let Some(weather) = weather_state {
        let is_dangerous = matches!(
            weather.current_weather,
            WeatherType::MagneticStorm
                | WeatherType::ThermalInversion
                | WeatherType::Storm
                | WeatherType::MutagenicRain
        );

        if is_dangerous {
            for (pos, mut stress) in pops.iter_mut() {
                let mut is_outside = true;
                if let Some(ref roofs) = roof_grid {
                    if roofs.has_roof(pos.x, pos.y) {
                        is_outside = false;
                    }
                }

                if is_outside {
                    stress.accumulated_stress += WEATHER_STRESS_PENALTY;
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(weather_madness_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_weather_madness_increases_stress() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, StressTracker::default()))
            .id();

        world.run_system_once(weather_madness_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }
}
