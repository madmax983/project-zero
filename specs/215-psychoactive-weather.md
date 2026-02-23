# 215: Psychoactive Weather

## Overview

Expand the weather system to include rare, "psychoactive" events that affect Pop psychology and physiology directly. These events add narrative depth and challenge by introducing temporary colony-wide mood and efficiency modifiers that players must adapt to.

New Weather Types:
- **Gloom Fog**: Thick, oppressive fog. Causes sadness (-Mood) and slows movement.
- **Manic Wind**: Charged static winds. Increases energy (+Work Speed) but also hunger (+Hunger Decay) and irritability.
- **Spore Storm**: Alien pollen. Induces vivid dreams (+Artistic/Research XP - future) but reduces focus (-Work Efficiency) and causes mild sickness risk.
- **Void Static**: Electromagnetic anomaly. Causes headaches (-Mood) and potential tech glitches (future).

## Dependencies

- `079` — Weather Events (Core system)
- `019` — Pop Thoughts & Mood (Morale system)
- `005` — Pop Needs (Hunger decay)
- `004` — Pop Entity (Speed)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/weather.rs - New tests module

#[cfg(test)]
mod psychoactive_tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::{Pop, Speed};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_psychoactive_weather_types_exist() {
        // Verify new variants are defined
        let _ = WeatherType::GloomFog;
        let _ = WeatherType::ManicWind;
        let _ = WeatherType::SporeStorm;
        let _ = WeatherType::VoidStatic;
    }

    #[test]
    fn test_psychoactive_state_application() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::GloomFog,
            duration_remaining: 100,
        });

        // Spawn a pop without PsychoactiveState
        let pop = world.spawn(Pop).id();

        // Run the system that applies state
        // (Note: You'll need to register/expose this system)
        bevy_ecs::system::RunSystemOnce::run_system_once(
            apply_psychoactive_weather_system,
            &mut world
        );

        // Verify component was added
        assert!(world.get::<PsychoactiveState>(pop).is_some());

        let state = world.get::<PsychoactiveState>(pop).unwrap();
        // GloomFog should reduce mood
        assert!(state.mood_modifier < 0.0);
    }

    #[test]
    fn test_manic_wind_effects() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::ManicWind,
            duration_remaining: 100,
        });

        let pop = world.spawn(Pop).id();
        bevy_ecs::system::RunSystemOnce::run_system_once(
            apply_psychoactive_weather_system,
            &mut world
        );

        let state = world.get::<PsychoactiveState>(pop).unwrap();
        // Manic Wind: +Work Speed, +Hunger Decay
        assert!(state.work_speed_modifier > 1.0);
        assert!(state.hunger_decay_modifier > 1.0);
    }

    #[test]
    fn test_spore_storm_effects() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::SporeStorm,
            duration_remaining: 100,
        });

        let pop = world.spawn(Pop).id();
        bevy_ecs::system::RunSystemOnce::run_system_once(
            apply_psychoactive_weather_system,
            &mut world
        );

        let state = world.get::<PsychoactiveState>(pop).unwrap();
        // Spore Storm: -Work Speed (loss of focus)
        assert!(state.work_speed_modifier < 1.0);
    }

    #[test]
    fn test_clear_weather_removes_state() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        let pop = world.spawn((
            Pop,
            PsychoactiveState {
                mood_modifier: -0.1,
                work_speed_modifier: 0.9,
                hunger_decay_modifier: 1.0,
            }
        )).id();

        bevy_ecs::system::RunSystemOnce::run_system_once(
            apply_psychoactive_weather_system,
            &mut world
        );

        // Component should be removed or reset to neutral
        // Removing is cleaner
        assert!(world.get::<PsychoactiveState>(pop).is_none());
    }

    // Integration Test for Needs (requires mocking or integration setup)
    #[test]
    fn test_needs_decay_with_psychoactive_modifier() {
        use crate::layer1::needs::{decay_needs_system, Needs};

        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Spawn pop with Manic Wind state (High Hunger Decay)
        let pop = world.spawn((
            Pop,
            Needs::default(),
            PsychoactiveState {
                hunger_decay_modifier: 2.0, // Double decay
                ..Default::default()
            }
        )).id();

        // Run decay
        bevy_ecs::system::RunSystemOnce::run_system_once(
            decay_needs_system,
            &mut world
        );

        let needs = world.get::<Needs>(pop).unwrap();
        // Normal decay is 0.001. With 2.0x, it should be 0.002.
        // 0.8 - 0.002 = 0.798
        assert!((needs.hunger - 0.798).abs() < 0.0001);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `WeatherType`

```rust
// src/layer1/weather.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    #[default]
    Clear,
    Rain,
    Storm,
    Fog,
    Heatwave,
    Snow,
    ThermalInversion,
    MagneticStorm,
    // New Types
    GloomFog,
    ManicWind,
    SporeStorm,
    VoidStatic,
}

impl WeatherType {
    // ... existing impls ...

    pub fn speed_modifier(&self) -> f32 {
        match self {
            // ... existing ...
            Self::GloomFog => 0.6,
            Self::ManicWind => 1.1, // Windy but energetic
            Self::SporeStorm => 0.8,
            Self::VoidStatic => 1.0,
            _ => 1.0, // Fallback
        }
    }
}
```

### 2. Define `PsychoactiveState` Component

```rust
// src/layer1/weather.rs

#[derive(Component, Debug, Clone, Copy)]
pub struct PsychoactiveState {
    pub mood_modifier: f32,
    pub work_speed_modifier: f32,
    pub hunger_decay_modifier: f32,
}

impl Default for PsychoactiveState {
    fn default() -> Self {
        Self {
            mood_modifier: 0.0,
            work_speed_modifier: 1.0,
            hunger_decay_modifier: 1.0,
        }
    }
}
```

### 3. Implement `apply_psychoactive_weather_system`

```rust
// src/layer1/weather.rs

use crate::layer1::morale::{Morale, MoodModifier};

pub fn apply_psychoactive_weather_system(
    mut commands: Commands,
    weather: Res<WeatherState>,
    // Query pops that might need update
    // Added<Pop> or Changed<WeatherState> logic is ideal,
    // but for MVP iterating all pops is acceptable (or use a change detection resource).
    pop_query: Query<(Entity, Option<&PsychoactiveState>), With<Pop>>,
) {
    let (mood_mod, work_mod, hunger_mod, name) = match weather.current_weather {
        WeatherType::GloomFog => (-0.15, 0.9, 1.0, "Gloom Fog"),
        WeatherType::ManicWind => (0.1, 1.2, 1.5, "Manic Wind"), // Fast work, hungry
        WeatherType::SporeStorm => (-0.05, 0.7, 1.0, "Spore Dreams"), // Distracted
        WeatherType::VoidStatic => (-0.2, 1.0, 1.0, "Void Headache"),
        _ => {
            // Clear or normal weather: Remove component
            for (entity, state) in pop_query.iter() {
                if state.is_some() {
                    commands.entity(entity).remove::<PsychoactiveState>();
                }
            }
            return;
        }
    };

    for (entity, state) in pop_query.iter() {
        let needs_update = if let Some(current) = state {
             (current.mood_modifier - mood_mod).abs() > f32::EPSILON ||
             (current.work_speed_modifier - work_mod).abs() > f32::EPSILON
        } else {
            true
        };

        if needs_update {
            commands.entity(entity).insert(PsychoactiveState {
                mood_modifier: mood_mod,
                work_speed_modifier: work_mod,
                hunger_decay_modifier: hunger_mod,
            });

            // Also apply immediate MoodModifier to Morale component for display/UI
            // Note: This logic might duplicate every tick if not careful.
            // Better to let Morale system read PsychoactiveState directly in REFACTOR,
            // or just rely on the component for logic.
            // For now, PsychoactiveState is the source of truth for modifiers.
        }
    }
}
```

### 4. Update `decay_needs_system`

```rust
// src/layer1/needs.rs

// Add PsychoactiveState to query
use crate::layer1::weather::PsychoactiveState;

pub fn decay_needs_system(
    mut query: Query<(&mut Needs, Option<&Traits>, Option<&PsychoactiveState>), Without<crate::layer1::cryo::CryoStasis>>,
    policies: Option<Res<ColonyPolicies>>,
) {
    let hunger_mod = policies.map_or(1.0, |p| get_hunger_decay_modifier(&p));
    let base_hunger_decay = HUNGER_DECAY_PER_TICK * hunger_mod;

    query.par_iter_mut().for_each(|(mut needs, traits, weather_state)| {
        let trait_mod = traits.map_or(1.0, get_trait_hunger_decay_modifier);
        let weather_mod = weather_state.map_or(1.0, |w| w.hunger_decay_modifier);

        let hunger_decay = base_hunger_decay * trait_mod * weather_mod;

        needs.hunger = (needs.hunger - hunger_decay).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
        needs.leisure = (needs.leisure - LEISURE_DECAY_PER_TICK).max(0.0);
    });
}
```

### 5. Update `work_execution_system` / `calculate_work_amount`

```rust
// src/layer1/execution/general_work.rs

// In work_execution_system query:
// Add Option<&PsychoactiveState>

// Inside map/processing loop:
// let weather_mod = psychoactive_state.map_or(1.0, |s| s.work_speed_modifier);
// Combine into `local_mod` passed to process_single_worker.
```

### 6. Integrate with Morale Display

In `src/layer1/morale.rs`, update `update_morale_cache_system` to read `PsychoactiveState` and add its `mood_modifier` to the sum.

## REFACTOR Phase: Quality & Design

- **Event-Driven Application**: Instead of polling `WeatherState` every tick, use `Changed<WeatherState>` filter in `apply_psychoactive_weather_system`.
- **System Ordering**: Ensure `apply_psychoactive_weather_system` runs before `decay_needs_system` and `work_execution_system`.
- **Config**: Move magic numbers (0.1, 1.2, etc.) to a config file or consts.
- **Visuals**: Add specific particle effects for `GloomFog` (gray particles) or `SporeStorm` (green particles).

## Acceptance Criteria

- [ ] All RED tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `GloomFog` reduces morale.
- [ ] `ManicWind` increases work speed and hunger decay.
- [ ] `SporeStorm` reduces work speed.
- [ ] Weather changes correctly apply/remove `PsychoactiveState`.
- [ ] Needs decay logic includes weather modifier.
- [ ] Work calculation includes weather modifier.

## Technical Guidance

- **Circular Dependencies**: `needs.rs` needs `PsychoactiveState` from `weather.rs`. `weather.rs` needs `Pop` from `pop.rs`. `pop.rs` needs `Needs` from `needs.rs`. This is fine (struct dependencies), but be careful with module imports.
- **System Registration**: Don't forget to register `apply_psychoactive_weather_system` in `main.rs`.
- **Serialization**: `PsychoactiveState` should derive `Serialize, Deserialize` if save/load is implemented (it is).
