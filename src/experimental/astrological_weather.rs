//! Astrological Weather (Stargazing Leisure) system.
//!
//! This module connects the `AstrologicalBelief` system to the `WeatherType` system.
//! Pops who believe in astrology look to the skies for comfort and guidance.
//! During `WeatherType::Clear` weather, they can see the stars and naturally regenerate
//! their `leisure` need (stargazing).
//! However, if the stars are completely obscured by `WeatherType::Fog` or `WeatherType::Storm`,
//! they suffer from "astrological disconnect" and their `leisure` decays slightly faster.
//!
//! Pops with the `Rationalist` trait ignore these effects entirely.

use crate::layer1::culture::astrology::{AstrologicalBelief, Rationalist};
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

/// Adjusts the leisure of pops with `AstrologicalBelief` based on the weather.
pub fn stargazing_leisure_system(
    weather: Res<WeatherState>,
    mut query: Query<(&AstrologicalBelief, &mut Needs), Without<Rationalist>>,
) {
    let leisure_change = match weather.current_weather {
        WeatherType::Clear => 0.005,                     // Small passive gain
        WeatherType::Fog | WeatherType::Storm => -0.005, // Small passive penalty
        _ => 0.0,
    };

    if leisure_change == 0.0 {
        return;
    }

    for (_, mut needs) in query.iter_mut() {
        needs.leisure = (needs.leisure + leisure_change).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_stargazing_leisure_buffs_during_clear_weather() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        let entity = world
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: false,
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(stargazing_leisure_system).unwrap();

        let needs = world.get::<Needs>(entity).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Leisure should increase during clear weather"
        );
    }

    #[test]
    fn test_stargazing_leisure_debuffs_during_storm() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Storm,
            duration_remaining: 100,
        });

        let entity = world
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: false,
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(stargazing_leisure_system).unwrap();

        let needs = world.get::<Needs>(entity).unwrap();
        assert!(needs.leisure < 0.5, "Leisure should decrease during storm");
    }

    #[test]
    fn test_stargazing_leisure_ignored_by_rationalists() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        let entity = world
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: false,
                },
                Rationalist,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(stargazing_leisure_system).unwrap();

        let needs = world.get::<Needs>(entity).unwrap();
        assert_eq!(needs.leisure, 0.5, "Rationalists should not be affected");
    }

    #[test]
    fn test_stargazing_leisure_neutral_weather() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain, // Neutral
            duration_remaining: 100,
        });

        let entity = world
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: false,
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(stargazing_leisure_system).unwrap();

        let needs = world.get::<Needs>(entity).unwrap();
        assert_eq!(
            needs.leisure, 0.5,
            "Neutral weather should not affect leisure"
        );
    }
}
