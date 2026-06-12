//! Emotional Weather (Nova Feature).
//!
//! # The Spark
//! We have a weather system and a morale system, but they don't interact.
//! What if the collective emotional state of the colony could manifest as literal weather?
//!
//! # The Feature
//! The `emotional_weather_system` calculates the average morale of the colony.
//! If the colony is extremely depressed, it will start to rain or storm.
//! If the colony is extremely happy, the skies will clear.

use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::weather::{WeatherState, WeatherType};
use bevy_ecs::prelude::*;

/// System that checks the average morale of the colony and forces weather changes.
pub fn emotional_weather_system(
    pops: Query<&Morale, With<Pop>>,
    weather_state: Option<ResMut<WeatherState>>,
) {
    if pops.is_empty() {
        return;
    }

    if let Some(mut state) = weather_state {
        let mut total_morale = 0.0;
        let mut count = 0;

        for morale in pops.iter() {
            total_morale += morale.value;
            count += 1;
        }

        let avg_morale = total_morale / count as f32;

        // If very depressed, make it storm
        if avg_morale < 0.2 && state.current_weather != WeatherType::Storm {
            state.current_weather = WeatherType::Storm;
            state.duration_remaining = 100; // Force it for a while
        }
        // If very happy, clear skies
        else if avg_morale > 0.8 && state.current_weather != WeatherType::Clear {
            state.current_weather = WeatherType::Clear;
            state.duration_remaining = 100;
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(emotional_weather_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_emotional_weather_storm_on_low_morale() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 10,
        });

        // Add depressed pop
        world.spawn((
            Pop,
            Morale {
                value: 0.1,
                ..Default::default()
            },
        ));

        world.run_system_once(emotional_weather_system).unwrap();

        let weather = world.resource::<WeatherState>();
        assert_eq!(weather.current_weather, WeatherType::Storm);
        assert_eq!(weather.duration_remaining, 100);
    }

    #[test]
    fn test_emotional_weather_clear_on_high_morale() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            duration_remaining: 10,
        });

        // Add happy pop
        world.spawn((
            Pop,
            Morale {
                value: 0.9,
                ..Default::default()
            },
        ));

        world.run_system_once(emotional_weather_system).unwrap();

        let weather = world.resource::<WeatherState>();
        assert_eq!(weather.current_weather, WeatherType::Clear);
        assert_eq!(weather.duration_remaining, 100);
    }

    #[test]
    fn test_emotional_weather_no_change_on_average_morale() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            duration_remaining: 10,
        });

        // Add average pop
        world.spawn((
            Pop,
            Morale {
                value: 0.5,
                ..Default::default()
            },
        ));

        world.run_system_once(emotional_weather_system).unwrap();

        let weather = world.resource::<WeatherState>();
        assert_eq!(weather.current_weather, WeatherType::Rain); // No change
        assert_eq!(weather.duration_remaining, 10);
    }
}
