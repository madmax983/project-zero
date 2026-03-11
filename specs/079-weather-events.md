# 079: Weather Events

## Overview

Implement a dynamic weather system where weather conditions (Clear, Rain, Storm, Fog, Heatwave, Snow) change over time based on the current season. Weather affects pop movement speed, agricultural productivity (future), and provides visual feedback (future).

## Dependencies

- `027` — Seasonal Rhythms (provides `Season` and `SeasonState`)
- `010` — Chronicle System (provides `Chronicle` for logging)
- `001` — SimulationTime (provides `tick`)
- `004` — Pop Entity (provides `Speed` component)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/weather.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::shared::time::SimulationTime;
    use crate::layer1::chronicle::Chronicle;
    use crate::layer1::pop::{Pop, Speed};
    use bevy_ecs::prelude::*;

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

        update_weather_system(&mut world);

        let state = world.resource::<WeatherState>();
        assert_eq!(state.duration_remaining, initial_duration - 1);
    }

    #[test]
    fn test_update_weather_system_changes_weather() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 1, // Will hit 0
        });
        world.insert_resource(SeasonState {
            current_season: Season::Winter, // Force Winter for predictable snow (mocking RNG needed?)
            ..Default::default()
        });
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());

        // We can't easily mock RNG in system tests without dependency injection,
        // but we can verify the state changes.
        // For deterministic testing, we might need a seeded RNG resource or check that *some* valid weather is picked.

        update_weather_system(&mut world);

        let state = world.resource::<WeatherState>();
        assert!(state.duration_remaining > 0);
        // Chronicle should have an event
        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
    }

    #[test]
    fn test_apply_weather_effects_system_slows_movement() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Storm,
            duration_remaining: 100,
        });

        let pop = world.spawn((
            Pop,
            Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        )).id();

        bevy_ecs::system::RunSystemOnce::run_system_once(apply_weather_effects_system, &mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        // Storm should reduce speed to 0.5
        assert!((speed.current - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_weather_effects_system_resets_speed() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        let pop = world.spawn((
            Pop,
            Speed {
                base: 1.0,
                current: 0.5, // Was slowed
                accumulator: 0.0,
            },
        )).id();

        bevy_ecs::system::RunSystemOnce::run_system_once(apply_weather_effects_system, &mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        // Clear weather should restore to base
        assert!((speed.current - 1.0).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Weather Types and Resource

```rust
// src/layer1/weather.rs

use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::seasons::{Season, SeasonState};
use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer1::pop::Speed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    #[default]
    Clear,
    Rain,
    Storm,
    Fog,
    Heatwave,
    Snow,
}

impl WeatherType {
    pub fn name(&self) -> &'static str {
        match self {
            WeatherType::Clear => "Clear Skies",
            WeatherType::Rain => "Rain",
            WeatherType::Storm => "Storm",
            WeatherType::Fog => "Fog",
            WeatherType::Heatwave => "Heatwave",
            WeatherType::Snow => "Snow",
        }
    }

    pub fn speed_modifier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::Rain => 0.8,
            WeatherType::Storm => 0.5,
            WeatherType::Fog => 0.7,
            WeatherType::Heatwave => 0.9,
            WeatherType::Snow => 0.6,
        }
    }
}

#[derive(Resource)]
pub struct WeatherState {
    pub current_weather: WeatherType,
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
        let tick = world.resource::<crate::shared::time::SimulationTime>().tick;
        world.resource_mut::<Chronicle>().add_event(
            tick,
            format!("Weather changed to {}.", new_weather.name()),
            EventImportance::Minor,
        );
    }
}

fn pick_weather_for_season(season: Season, rng: &mut impl Rng) -> WeatherType {
    let roll = rng.gen_range(0.0..1.0);
    match season {
        Season::Spring => {
            if roll < 0.6 { WeatherType::Clear }
            else if roll < 0.9 { WeatherType::Rain }
            else { WeatherType::Fog }
        },
        Season::Summer => {
            if roll < 0.7 { WeatherType::Clear }
            else if roll < 0.85 { WeatherType::Heatwave }
            else { WeatherType::Storm }
        },
        Season::Autumn => {
            if roll < 0.5 { WeatherType::Clear }
            else if roll < 0.8 { WeatherType::Rain }
            else { WeatherType::Fog }
        },
        Season::Winter => {
            if roll < 0.4 { WeatherType::Clear }
            else if roll < 0.9 { WeatherType::Snow }
            else { WeatherType::Storm } // Blizzard
        },
    }
}

pub fn apply_weather_effects_system(
    mut pops: Query<&mut Speed>,
    weather: Res<WeatherState>,
) {
    let modifier = weather.current_weather.speed_modifier();
    for mut speed in &mut pops {
        speed.current = speed.base * modifier;
    }
}
```

### 2. Registration

- Add `pub mod weather;` to `src/layer1/mod.rs`.
- Register `WeatherState` and systems in `src/main.rs`.
- `update_weather_system` runs in `SimulationSchedule`.
- `apply_weather_effects_system` runs after `update_weather_system` but before `movement_system`.

## REFACTOR Phase: Quality & Design

- **Config**: Move weather probabilities and duration ranges to a config file.
- **Visuals**: Add `WeatherOverlay` resource to render rain/snow characters on the map.
- **Events**: Fire a Bevy Event `WeatherChanged` so other systems can react without polling.
- **Optimization**: `apply_weather_effects_system` iterates all pops every tick. It could be optimized to run only on weather change, but checking every tick ensures newly spawned pops get the effect. Better: `Changed<WeatherState>` filter + `Added<Pop>` filter.

## Acceptance Criteria

- [ ] `WeatherState` resource tracks current weather.
- [ ] Weather changes periodically based on Season.
- [ ] Chronicle logs weather changes.
- [ ] Pop movement speed is affected by weather (e.g., Storm slows to 50%).
- [ ] `cargo test` passes with 85%+ coverage.

## Questions

- Should weather affect crop yield immediately or just via seasonal modifier? (Stick to seasonal modifier for now, maybe add `CropDamage` event for storms later).
- Should extreme weather (Heatwave/Blizzard) cause damage? (Future spec: `080-extreme-weather-damage`).

*Architect:* Use seasonal modifiers for the MVP. Immediate crop damage from storms will be a future enhancement.
