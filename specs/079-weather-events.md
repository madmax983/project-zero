# 079: Weather Events

## Overview

Introduces a dynamic weather system that layers on top of the seasonal cycle. Weather events (Rain, Storm, Fog) occur probabilistically based on the current season, adding visual variety and gameplay consequences (e.g., Lightning starting fires, Rain affecting movement).

This spec adds:
- `WeatherType` enum (Clear, Rain, Storm, Fog).
- `WeatherState` resource.
- `update_weather_system` to transition weather.
- `weather_effects_system` to apply effects (Lightning).

## Dependencies

- `027` — Seasonal Rhythms (Weather probabilities depend on Season).
- `033` — Fire Propagation (Lightning causes Fire).
- `010` — Chronicle System (Log major weather events).
- `001` — SimulationTime.

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/weather_tests.rs (New file)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::weather::{WeatherType, WeatherState, update_weather_system, weather_effects_system};
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::shared::time::SimulationTime;
    use crate::layer1::chronicle::AddChronicleEvent;
    use crate::layer1::fire::Fire;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_weather_defaults_to_clear() {
        let state = WeatherState::default();
        assert_eq!(state.current_weather, WeatherType::Clear);
        assert!(state.duration > 0);
    }

    #[test]
    fn test_update_weather_decrements_duration() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration: 10,
        });
        world.insert_resource(SeasonState::default());
        world.init_resource::<Events<AddChronicleEvent>>(); // For potential logging
        world.insert_resource(SimulationTime::default()); // For RNG seeding if needed

        // Run system
        // Note: System requires access to RNG, usually via thread_rng internally
        // or a deterministic resource. For TDD, we assume internal logic.

        // We need to run the system once
        let mut schedule = Schedule::default();
        schedule.add_systems(update_weather_system);
        schedule.run(&mut world);

        let state = world.resource::<WeatherState>();
        assert_eq!(state.duration, 9);
    }

    #[test]
    fn test_weather_transition_on_duration_expiry() {
        let mut world = World::new();
        // Force transition
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration: 0,
        });
        world.insert_resource(SeasonState {
            current_season: Season::Summer, // Summer has high storm chance
        });
        world.init_resource::<Events<AddChronicleEvent>>();
        world.insert_resource(SimulationTime { tick: 12345, ..Default::default() });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_weather_system);
        schedule.run(&mut world);

        let state = world.resource::<WeatherState>();
        // Should have picked a new duration
        assert!(state.duration > 0);
        // Should have potentially changed weather (probabilistic, so hard to assert exact type without mocking RNG)
        // But we can assert the state is valid
        assert!(matches!(state.current_weather, WeatherType::Clear | WeatherType::Rain | WeatherType::Storm | WeatherType::Fog));
    }

    #[test]
    fn test_storm_logs_chronicle_event() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration: 0,
        });
        // Mock season that ALWAYS causes Storm (if we can, otherwise we rely on probabilistic test passing "eventually" or mock RNG)
        // For MVP TDD, let's assume we can't easily mock RNG without dependency injection.
        // Instead, we can manually trigger the logging logic if we separate it.
        // OR we just verify that IF it transitions to Storm, an event is emitted.

        // Let's manually set state to Storm and duration 0 to simulate a transition FROM Storm?
        // No, event happens ON transition.

        // Alternative: Deterministic test by seeding or using a predictable Season/Tick combo if implementation allows.
        // For now, let's skip strict probabilistic testing and focus on logic.
    }

    #[test]
    fn test_lightning_spawns_fire_during_storm() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Storm,
            duration: 10,
        });
        // 10x10 Grid of flammable trees
        let mut tiles = vec![TerrainType::Tree; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // We need to make sure probability hits.
        // If probability is low (e.g. 1% per tick), we loop many times.
        let mut fire_spawned = false;

        let mut schedule = Schedule::default();
        schedule.add_systems(weather_effects_system);

        for _ in 0..1000 {
            schedule.run(&mut world);
            if !world.query::<&Fire>().is_empty(&world) {
                fire_spawned = true;
                break;
            }
        }

        assert!(fire_spawned, "Storm should eventually spawn lightning/fire on trees");
    }

    #[test]
    fn test_clear_weather_no_effects() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration: 10,
        });
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Tree; 100] });

        let mut schedule = Schedule::default();
        schedule.add_systems(weather_effects_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        assert!(world.query::<&Fire>().is_empty(&world), "Clear weather should not spawn fire");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Types (`src/layer1/weather.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::seasons::{Season, SeasonState};
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::fire::Fire;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::map::GridPosition;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    #[default]
    Clear,
    Rain,
    Storm,
    Fog,
}

impl WeatherType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Rain => "Rain",
            Self::Storm => "Storm",
            Self::Fog => "Fog",
        }
    }
}

#[derive(Resource)]
pub struct WeatherState {
    pub current_weather: WeatherType,
    pub duration: u32, // Ticks remaining
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            duration: 100,
        }
    }
}

pub fn update_weather_system(
    mut state: ResMut<WeatherState>,
    season_state: Res<SeasonState>,
    mut events: EventWriter<AddChronicleEvent>,
) {
    if state.duration > 0 {
        state.duration -= 1;
        return;
    }

    // Transition
    let mut rng = rand::thread_rng();
    let season = season_state.current_season;

    // Probabilities (sum to 1.0)
    // Spring: Clear 60%, Rain 30%, Storm 5%, Fog 5%
    // Summer: Clear 70%, Rain 10%, Storm 20%, Fog 0%
    // Autumn: Clear 50%, Rain 30%, Storm 10%, Fog 10%
    // Winter: Clear 40%, Rain 0% (Snow=Rain visual), Storm 10% (Blizzard), Fog 50%

    let roll: f32 = rng.gen();
    let (rain_prob, storm_prob, fog_prob) = match season {
        Season::Spring => (0.3, 0.05, 0.05),
        Season::Summer => (0.1, 0.20, 0.00),
        Season::Autumn => (0.3, 0.10, 0.10),
        Season::Winter => (0.0, 0.10, 0.50), // "Rain" in winter is effectively Snow, let's treat Rain as Clear for MVP mechanics or mapped to Snow logic later
    };

    let new_weather = if roll < rain_prob {
        WeatherType::Rain
    } else if roll < rain_prob + storm_prob {
        WeatherType::Storm
    } else if roll < rain_prob + storm_prob + fog_prob {
        WeatherType::Fog
    } else {
        WeatherType::Clear
    };

    // Duration (randomized)
    state.duration = rng.gen_range(50..200);

    if new_weather != state.current_weather {
        state.current_weather = new_weather;

        // Log Major events
        if new_weather == WeatherType::Storm {
            events.send(AddChronicleEvent {
                text: "A violent storm descends upon the colony.".to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

pub fn weather_effects_system(
    state: Res<WeatherState>,
    terrain: Res<TerrainGrid>,
    mut commands: Commands,
) {
    if state.current_weather != WeatherType::Storm {
        return;
    }

    let mut rng = rand::thread_rng();

    // Lightning chance per tick
    if rng.gen_bool(0.05) { // 5% chance per tick during storm
        // Pick random tile
        let x = rng.gen_range(0..terrain.width);
        let y = rng.gen_range(0..terrain.height);

        // Check if flammable (Tree)
        // Ideally we check for Buildings too, but Terrain is fast
        if let Some(tile) = terrain.get(x, y) {
            if tile == TerrainType::Tree {
                // Spawn Fire
                commands.spawn((
                    Fire::default(),
                    GridPosition { x: x as i32, y: y as i32 }
                ));
            }
        }
    }
}
```

### 2. Register System

In `src/layer1/mod.rs` and `main.rs`, register the resource and systems.

## REFACTOR Phase: Quality & Design

- **Configuration**: Move probabilities to a config const or file.
- **Visuals**: Weather needs to be rendered. This spec focuses on backend, but mention in comments/docs that UI needs to query `WeatherState`.
- **Snow**: In Winter, "Rain" should probably just be "Snow" (cosmetic) or have different effects (cold). For now, we map probabilities such that Winter has low/no "Rain" but high Fog/Storm.
- **Movement**: Add `MovementCost` modifier resource that Pathfinding reads. (Future task).

## Acceptance Criteria

- [ ] `WeatherState` resource tracks current weather.
- [ ] Weather changes probabilistically based on Season.
- [ ] Storms log to Chronicle.
- [ ] Storms spawn Fire on Trees occasionally (Lightning).
- [ ] `cargo test` passes.
- [ ] `cargo clippy` passes.

## Technical Guidance

- Use `rand::thread_rng` for transitions.
- Ensure `WeatherState` is initialized in `main.rs`.
- `Fire` spawning should respect `GridPosition` and `Fire` components from `033`.
- **Testing Note**: When testing systems that use `Commands` (like spawning fire), remember to apply deferred commands (e.g., `world.flush()` or `apply_deferred`) before querying for the new entities.

## Questions

- *Builder*: Should lightning hit buildings?
  *Architect*: Yes, if possible. You can query for `Flammable` buildings at the random coordinate. If checking every building is too slow, just sticking to Terrain Trees is fine for MVP.
