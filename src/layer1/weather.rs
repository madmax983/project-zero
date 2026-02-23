//! Weather system for the colony.
//!
//! Handles:
//! - Weather state (Clear, Rain, Storm, etc.)
//! - Weather transitions based on Seasons.
//! - Effects on pop speed.

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::pop::Speed;
use crate::layer1::seasons::{Season, SeasonState};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Types of weather conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    /// No adverse weather.
    #[default]
    Clear,
    /// Light rain, minor slowdown.
    Rain,
    /// Heavy storm, major slowdown.
    Storm,
    /// Fog, reduces visibility (future) and speed.
    Fog,
    /// Extreme heat, minor slowdown.
    Heatwave,
    /// Snow, moderate slowdown.
    Snow,
    /// Thermal Inversion, traps smog.
    ThermalInversion,
    /// Magnetic Storm, high energy interference.
    MagneticStorm,
}

impl WeatherType {
    /// Returns the display name of the weather.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Clear => "Clear Skies",
            Self::Rain => "Rain",
            Self::Storm => "Storm",
            Self::Fog => "Fog",
            Self::Heatwave => "Heatwave",
            Self::Snow => "Snow",
            Self::ThermalInversion => "Thermal Inversion",
            Self::MagneticStorm => "Magnetic Storm",
        }
    }

    /// Returns the speed modifier for this weather (1.0 = normal).
    #[must_use]
    pub const fn speed_modifier(&self) -> f32 {
        match self {
            Self::Clear | Self::ThermalInversion => 1.0,
            Self::Rain => 0.8,
            Self::Storm => 0.5,
            Self::Fog | Self::MagneticStorm => 0.7,
            Self::Heatwave => 0.9,
            Self::Snow => 0.6,
        }
    }
}

/// Resource tracking the current weather state.
#[derive(Resource)]
pub struct WeatherState {
    /// The current active weather.
    pub current_weather: WeatherType,
    /// Ticks remaining for the current weather.
    pub duration_remaining: u32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            duration_remaining: 100, // Initial buffer
        }
    }
}

/// System to update weather duration and change weather when it expires.
pub fn update_weather_system(world: &mut World) {
    // 1. Check/update duration (scope the mutable borrow)
    let time_to_change = {
        let mut state = world.resource_mut::<WeatherState>();
        if state.duration_remaining > 0 {
            state.duration_remaining -= 1;
            false
        } else {
            true
        }
    };

    if !time_to_change {
        return;
    }

    // 2. Determine new weather
    let season = world.resource::<SeasonState>().current_season;
    let mut rng = rand::thread_rng();
    let new_weather = pick_weather_for_season(season, &mut rng);
    let new_duration = rng.gen_range(50..200);

    // 3. Apply changes and log
    {
        let mut state = world.resource_mut::<WeatherState>();
        state.current_weather = new_weather;
        state.duration_remaining = new_duration;
    }

    if new_weather != WeatherType::Clear {
        // We can't access SimulationTime inside the exclusive system easily if we also borrow World?
        // Wait, World has everything.
        // But to send event we need EventWriter logic or direct world access.
        // We can use world.send_event if we have the event type.

        let event = AddChronicleEvent {
            text: format!("Weather changed to {}.", new_weather.name()),
            importance: EventImportance::Minor,
        };
        world.send_event(event);
    }
}

fn pick_weather_for_season(season: Season, rng: &mut impl Rng) -> WeatherType {
    let roll = rng.gen_range(0.0..1.0);
    match season {
        Season::Spring => {
            if roll < 0.6 {
                WeatherType::Clear
            } else if roll < 0.9 {
                WeatherType::Rain
            } else {
                WeatherType::Fog
            }
        }
        Season::Summer => {
            if roll < 0.7 {
                WeatherType::Clear
            } else if roll < 0.85 {
                WeatherType::Heatwave
            } else {
                WeatherType::Storm
            }
        }
        Season::Autumn => {
            if roll < 0.5 {
                WeatherType::Clear
            } else if roll < 0.8 {
                WeatherType::Rain
            } else {
                WeatherType::Fog
            }
        }
        Season::Winter => {
            if roll < 0.4 {
                WeatherType::Clear
            } else if roll < 0.85 {
                WeatherType::Snow
            } else if roll < 0.95 {
                WeatherType::ThermalInversion
            } else {
                WeatherType::Storm
            } // Blizzard
        }
    }
}

/// Applies weather effects (speed penalties) to pops.
///
/// Multiplies `Speed.current` by the weather modifier.
/// Must run AFTER `apply_lighting_penalties_system` (which resets/sets base speed)
/// and BEFORE `movement_system`.
pub fn apply_weather_effects_system(mut pops: Query<&mut Speed>, weather: Res<WeatherState>) {
    let modifier = weather.current_weather.speed_modifier();
    // Optimization: Don't iterate if modifier is 1.0 (Clear)
    // However, if we skip 1.0, we rely on previous systems to have set the "clean" state.
    // Since this system is multiplicative, multiplying by 1.0 is safe.
    // If we skip, we just don't change anything, which is also correct (x * 1.0 = x).
    if (modifier - 1.0).abs() < f32::EPSILON {
        return;
    }

    for mut speed in &mut pops {
        speed.current *= modifier;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::Chronicle;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::shared::time::SimulationTime;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_weather_state_default() {
        let state = WeatherState::default();
        assert_eq!(state.current_weather, WeatherType::Clear);
        assert!(state.duration_remaining > 0);
    }

    #[test]
    fn test_update_weather_system_decrements_duration() {
        let mut world = World::new();
        let initial_duration = 100;
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: initial_duration,
        });
        world.insert_resource(SeasonState::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());
        // Need to add events resource for Chronicle to work
        world.init_resource::<Events<crate::layer1::chronicle::AddChronicleEvent>>();

        world.run_system_once(update_weather_system).unwrap();

        let state = world.resource::<WeatherState>();
        assert_eq!(state.duration_remaining, initial_duration - 1);
    }

    #[test]
    fn test_update_weather_system_changes_weather() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 0, // Force update immediately
        });
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());
        world.init_resource::<Events<crate::layer1::chronicle::AddChronicleEvent>>();

        // Run multiple times to statistically likely hit a non-Clear weather if random
        // Or mock RNG. Here we just check *if* it changes, or if logic runs.
        // For RED phase, just running it and checking state change or event is enough.
        // Since we can't mock RNG easily without dependency injection, we'll check logic flow.

        world.run_system_once(update_weather_system).unwrap();

        let state = world.resource::<WeatherState>();
        // Duration should be reset to > 0
        assert!(state.duration_remaining > 0);

        // We can't guarantee weather changed from Clear (it might pick Clear again),
        // but if it picked non-Clear, we should see an event.
        // Let's just check that duration reset, which implies logic ran.
    }

    #[test]
    fn test_apply_weather_effects_system_slows_movement() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Storm,
            duration_remaining: 100,
        });

        let pop = world
            .spawn((
                Pop,
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        world.run_system_once(apply_weather_effects_system).unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        // Storm modifier is 0.5. 1.0 * 0.5 = 0.5.
        assert!((speed.current - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_weather_effects_system_multiplies_speed() {
        // This test replaces `test_apply_weather_effects_system_resets_speed`
        // ensuring compatibility with lighting penalties.
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Storm, // 0.5 modifier
            duration_remaining: 100,
        });

        let pop = world
            .spawn((
                Pop,
                Speed {
                    base: 1.0,
                    current: 0.5, // Already slowed by darkness/lighting
                    accumulator: 0.0,
                },
            ))
            .id();

        world.run_system_once(apply_weather_effects_system).unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        // Should be 0.5 (current) * 0.5 (weather) = 0.25
        assert!(
            (speed.current - 0.25).abs() < f32::EPSILON,
            "Expected 0.25, got {}",
            speed.current
        );
    }
}
