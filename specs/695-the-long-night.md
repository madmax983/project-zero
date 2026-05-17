# 695: The Long Night

## Overview

A massive orbital event (such as a nebula, eclipse, or planetary alignment) blocks the sun for extended periods. During "The Long Night," solar power drops to zero, global temperatures plummet, and crops die unless grown indoors with artificial light. This creates a severe survival challenge, forcing the colony to rely on stored fuel, batteries, and bioluminescent flora.

## Dependencies

- `042` — Energy System
- `063` — Atmospheric Simulation
- `065` — Day/Night Cycle

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_long_night_event_disables_solar_power() {
        let mut app = App::new();

        app.insert_resource(EnergyGrid { available_solar: 100.0, total_demand: 50.0 });

        // Trigger The Long Night
        app.world_mut().send_event(StartLongNightEvent { duration_ticks: 10_000 });
        app.update();

        let event_active = app.world().resource::<LongNightEvent>();
        assert!(event_active.is_active);

        let grid = app.world().resource::<EnergyGrid>();
        // Solar power should be effectively 0
        assert_eq!(grid.available_solar, 0.0);
    }

    #[test]
    fn test_long_night_plummets_global_temperature() {
        let mut app = App::new();

        app.insert_resource(GlobalTemperature { current_temp: 20.0, base_temp: 20.0 });
        app.insert_resource(LongNightEvent { is_active: true, duration_remaining: 5000 });

        // Advance time and check temperature
        app.update();

        let temp = app.world().resource::<GlobalTemperature>();
        // Temperature should be drastically lower than base
        assert!(temp.current_temp < temp.base_temp - 15.0);
    }

    #[test]
    fn test_crops_die_during_long_night_without_light() {
        let mut app = App::new();

        let crop_id = app.world_mut().spawn((
            Crop { health: 100.0, requires_light: true },
            Transform::from_xyz(0.0, 0.0, 0.0), // No light source nearby
        )).id();

        app.insert_resource(LongNightEvent { is_active: true, duration_remaining: 5000 });

        // Advance time
        app.update();

        let crop = app.world().get::<Crop>(crop_id).unwrap();
        // Crop health should be decreasing
        assert!(crop.health < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct LongNightEvent {
    pub is_active: bool,
    pub duration_remaining: i32,
}

#[derive(Event)]
pub struct StartLongNightEvent {
    pub duration_ticks: i32,
}

#[derive(Resource, Default)]
pub struct EnergyGrid {
    pub available_solar: f32,
    pub total_demand: f32,
}

#[derive(Resource, Default)]
pub struct GlobalTemperature {
    pub current_temp: f32,
    pub base_temp: f32,
}

#[derive(Component)]
pub struct Crop {
    pub health: f32,
    pub requires_light: bool,
}

pub fn start_long_night(
    mut events: EventReader<StartLongNightEvent>,
    mut long_night: ResMut<LongNightEvent>,
) {
    for event in events.read() {
        long_night.is_active = true;
        long_night.duration_remaining = event.duration_ticks;
    }
}

pub fn process_long_night_effects(
    mut long_night: ResMut<LongNightEvent>,
    mut grid: ResMut<EnergyGrid>,
    mut temp: ResMut<GlobalTemperature>,
    mut crops: Query<&mut Crop>,
) {
    if long_night.is_active {
        // Decrease duration
        long_night.duration_remaining -= 1;
        if long_night.duration_remaining <= 0 {
            long_night.is_active = false;
        }

        // Disable solar power
        grid.available_solar = 0.0;

        // Plummet temperature
        temp.current_temp = -20.0; // Extreme cold

        // Kill crops requiring light
        for mut crop in crops.iter_mut() {
            if crop.requires_light {
                crop.health -= 1.0;
                if crop.health < 0.0 {
                    crop.health = 0.0;
                }
            }
        }
    } else {
        // Restore base temp slowly
        if temp.current_temp < temp.base_temp {
            temp.current_temp += 0.5;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Tie the `LongNightEvent` into the UI notifications system (a massive banner reading "The Long Night Begins").
- **Lighting**: Check for nearby artificial `LightSource` components on crops to protect them during the Long Night instead of indiscriminately killing them.
- **Morale**: Add a global `Morale` penalty for Pops experiencing the darkness, forcing them to huddle near light and heat sources.
- **Narrative**: Ensure a `ChronicleEvent` is triggered when the Long Night starts and ends, giving Pops a massive shared memory.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `StartLongNightEvent` sets the `LongNightEvent` active status and duration.
- [ ] While active, `available_solar` on the `EnergyGrid` drops to 0.0.
- [ ] Global temperatures are massively penalized while the event is active.
- [ ] Unprotected crops lose health during the event.

## Technical Guidance

- Ensure the `LongNightEvent` duration is decremented based on `SimulationTime` updates, not arbitrary frame ticks in the final implementation.
- This event should override the normal `Day/Night Cycle` lighting system completely (e.g., set the ambient global light strictly to darkness).

## Questions

*Builder: add questions here if spec is unclear.*
- **Architectural Contradictions:** `EnergyGrid` and `GlobalTemperature` do not exist in the codebase. Energy is managed differently and temperature is managed through `TemperatureGrid`, which is a spatial grid rather than a single global `GlobalTemperature` struct. The RED Phase tests and GREEN phase logic cannot be implemented as written. I'm moving on to a different task.
