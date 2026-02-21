# 192: Planetary Weather Fronts

## Overview

Introduces large-scale **Weather Fronts** that travel across the planet surface. Unlike random weather changes (Spec 079), Fronts are persistent, predictable entities that approach the colony, hit with defined duration and intensity, and pass.

This feature bridges the gap between Layer 2 (System/Planet scale) and Layer 1 (Colony scale) by providing strategic foresight. Players can see a "Dust Storm" coming 3 days in advance and prepare (harvest early, ground fleets, build shelters).

## Dependencies

- `079` — Weather Events (for `WeatherState`, `WeatherType`)
- `010` — Chronicle System (for logging forecasts)
- `001` — SimulationTime (for tracking movement)
- `027` — Seasonal Rhythms (for `SeasonState` used in spawning)

## RED Phase: Tests First

Write these tests in `src/layer1/weather_fronts_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::weather::{WeatherState, WeatherType};
    use crate::layer1::weather_fronts::{WeatherFront, WeatherFronts, update_weather_fronts_system};
    use crate::shared::time::SimulationTime;
    use crate::layer1::chronicle::Chronicle;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(WeatherFronts::default());
        world.insert_resource(WeatherState::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(Chronicle::default());
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

        // Mock time delta? Or assume system uses fixed time step.
        // For test, assume system advances by 1 tick = X hours.

        update_weather_fronts_system(&mut world);

        let fronts = world.resource::<WeatherFronts>();
        // 1000 - (100 * dt) < 1000
        assert!(fronts.active_fronts[0].distance_km < 1000.0, "Front should move closer");
    }

    #[test]
    fn test_front_arrival_overrides_weather() {
        let mut world = setup_world();

        // Front at 0 distance (arrived)
        let front = WeatherFront {
            weather_type: WeatherType::Storm,
            distance_km: 0.0,
            speed_km_h: 100.0,
            duration_h: 10.0,
        };

        world.resource_mut::<WeatherFronts>().active_fronts.push(front);

        // Ensure base weather is different
        world.resource_mut::<WeatherState>().current_weather = WeatherType::Clear;

        update_weather_fronts_system(&mut world);

        let weather = world.resource::<WeatherState>();
        assert_eq!(weather.current_weather, WeatherType::Storm, "Front should override weather");
        // Duration should be locked/set
        assert!(weather.duration_remaining > 0);
    }

    #[test]
    fn test_front_passing() {
        let mut world = setup_world();

        // Front active but duration expired
        let front = WeatherFront {
            weather_type: WeatherType::Storm,
            distance_km: 0.0,
            speed_km_h: 100.0,
            duration_h: 0.0, // Finished
        };

        world.resource_mut::<WeatherFronts>().active_fronts.push(front);
        // Current weather was storm
        world.resource_mut::<WeatherState>().current_weather = WeatherType::Storm;

        update_weather_fronts_system(&mut world);

        let fronts = world.resource::<WeatherFronts>();
        assert!(fronts.active_fronts.is_empty(), "Expired front should be removed");

        // Chronicle should log passing
        let chronicle = world.resource::<Chronicle>();
        assert!(chronicle.events.iter().any(|e| e.text.contains("passed")));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resources (`src/layer1/weather_fronts.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::weather::{WeatherState, WeatherType};
use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone)]
pub struct WeatherFront {
    pub weather_type: WeatherType,
    pub distance_km: f32, // Positive = approaching, 0 = active/passing
    pub speed_km_h: f32,  // Speed of approach
    pub duration_h: f32,  // Time remaining once active
}

#[derive(Resource, Default)]
pub struct WeatherFronts {
    pub active_fronts: Vec<WeatherFront>,
}

pub fn update_weather_fronts_system(world: &mut World) {
    // Separate scopes to avoid borrow conflicts
    let time_tick = world.resource::<SimulationTime>().tick;

    // Constant time step for MVP: 1 tick = 0.1 hours
    let dt_h = 0.1f32;

    let mut messages = Vec::new();
    let mut active_weather_override = None;

    world.resource_scope(|world, mut fronts: Mut<WeatherFronts>| {
        fronts.active_fronts.retain_mut(|front| {
            if front.distance_km > 0.0 {
                // Approaching
                front.distance_km -= front.speed_km_h * dt_h;
                if front.distance_km <= 0.0 {
                    front.distance_km = 0.0;
                    // Just arrived
                    messages.push((
                        format!("A {} front has hit the colony!", front.weather_type.name()),
                        EventImportance::Major
                    ));
                }
                true
            } else {
                // Active (distance == 0)
                front.duration_h -= dt_h;
                if front.duration_h > 0.0 {
                    active_weather_override = Some(front.weather_type);
                    true
                } else {
                    // Finished
                    messages.push((
                        format!("The {} has passed.", front.weather_type.name()),
                        EventImportance::Standard
                    ));
                    false // Remove
                }
            }
        });
    });

    // Apply Chronicle logs
    let mut chronicle = world.resource_mut::<Chronicle>();
    for (msg, importance) in messages {
        chronicle.add_event(time_tick, msg, importance);
    }

    // Apply Weather Override
    if let Some(new_type) = active_weather_override {
        let mut weather = world.resource_mut::<WeatherState>();
        weather.current_weather = new_type;
        // Lock duration so random system doesn't override it immediately
        weather.duration_remaining = 10;
    }
}
```

### 2. Spawner System (`spawn_weather_fronts_system`)

Occasionally spawns new fronts based on season. Runs less frequently (e.g., daily).

```rust
pub fn spawn_weather_fronts_system(
    mut fronts: ResMut<WeatherFronts>,
    season: Res<crate::layer1::seasons::SeasonState>,
) {
    // Random chance to spawn a front far away (e.g., 2000km)
    // Distance 2000km @ 50km/h = 40 hours warning
}
```

## REFACTOR Phase: Quality & Design

- **Forecast UI**: Add a UI panel showing "Approaching Fronts" with ETA.
  - ETA = `distance / speed`.
- **Layer 2 Visualization**: If `distance_km` is mapped to the planet map, show a sprite moving towards the colony pin.
- **Config**: Move speeds and durations to external config.
- **Interaction**: "Storm Shields" or "Harvest Rush" edicts could be triggered by these forecasts.

## Acceptance Criteria

- [ ] `WeatherFronts` resource implemented.
- [ ] Fronts move closer over time.
- [ ] Fronts override local weather when they arrive.
- [ ] Fronts disappear after duration.
- [ ] Chronicle logs arrival and departure.
- [ ] Tests pass.
