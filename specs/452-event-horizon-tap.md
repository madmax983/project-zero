# Specification 452: The Event Horizon Tap

## 1. Overview
This feature introduces a cross-layer interaction (Layer 2 -> Layer 1) where players can tether an experimental power station to a microscopic black hole or gravitational anomaly. It provides infinite energy to the colony, but the connection wavers. This causes localized "Time Dilation Zones" on the colony map where machines and Pops move at a fraction of their normal speed (e.g., 10%) but age and metabolize normally. This creates intense strategic tension during crises like pirate raids.

## 2. Dependencies
- `042` Energy System (`PowerSource` component)
- `065` Day/Night Cycle (or the Time system governing movement/work speed)
- `152` Orbital Stations (Layer 2 interaction)

## 3. RED Phase: Tests First

```rust
// tests/event_horizon_tap_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::energy::PowerSource;
    use scale::layer1::pop::MovementSpeed;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup time, grid, and systems
        world
    }

    #[test]
    fn test_event_horizon_tap_provides_infinite_energy() {
        let mut world = setup_world();

        let tap_entity = world.spawn((
            EventHorizonTap { is_active: true, instability: 0.0 },
            PowerSource { output: f32::INFINITY, active: true },
        )).id();

        let power_source = world.get::<PowerSource>(tap_entity).unwrap();
        assert_eq!(power_source.output, f32::INFINITY);
    }

    #[test]
    fn test_time_dilation_zone_slows_movement() {
        let mut world = setup_world();

        let tap_entity = world.spawn((
            EventHorizonTap { is_active: true, instability: 0.5 }, // Triggers zone
            GridPosition { x: 10, y: 10 },
        )).id();

        let pop = world.spawn((
            GridPosition { x: 10, y: 11 }, // Within radius
            MovementSpeed { base: 10.0, current: 10.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_time_dilation_system);
        schedule.run(&mut world);

        let speed = world.get::<MovementSpeed>(pop).unwrap();
        // Movement speed should be heavily reduced (e.g., 10% of base)
        assert_eq!(speed.current, 1.0);
    }

    #[test]
    fn test_time_dilation_outside_zone_unaffected() {
        let mut world = setup_world();

        let tap_entity = world.spawn((
            EventHorizonTap { is_active: true, instability: 0.5 },
            GridPosition { x: 10, y: 10 },
        )).id();

        let pop = world.spawn((
            GridPosition { x: 50, y: 50 }, // Far away
            MovementSpeed { base: 10.0, current: 10.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_time_dilation_system);
        schedule.run(&mut world);

        let speed = world.get::<MovementSpeed>(pop).unwrap();
        assert_eq!(speed.current, 10.0); // Unaffected
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/event_horizon_tap.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::MovementSpeed;
use crate::layer1::energy::PowerSource;

#[derive(Component, Debug, Clone)]
pub struct EventHorizonTap {
    pub is_active: bool,
    pub instability: f32, // 0.0 to 1.0
}

#[derive(Component, Debug, Clone)]
pub struct TimeDilationZone {
    pub radius: i32,
    pub speed_multiplier: f32,
}

pub fn apply_time_dilation_system(
    taps: Query<(&EventHorizonTap, &GridPosition)>,
    mut movers: Query<(&mut MovementSpeed, &GridPosition)>,
) {
    // Collect active zones based on instability
    let mut zones = Vec::new();
    for (tap, pos) in taps.iter() {
        if tap.is_active && tap.instability > 0.0 {
            // Radius scales with instability
            let radius = (tap.instability * 20.0) as i32;
            zones.push((*pos, radius, 0.1)); // 10% speed
        }
    }

    // Apply to movers
    for (mut speed, pos) in movers.iter_mut() {
        // Reset to base
        let mut current_mult = 1.0;

        for (zone_pos, radius, mult) in &zones {
            if pos.distance_chebyshev(*zone_pos) <= *radius {
                current_mult = current_mult.min(*mult);
            }
        }

        speed.current = speed.base * current_mult;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Instability Fluctuation**: Add a system that randomly shifts `instability` over time, perhaps influenced by Layer 2 space weather or system load.
- **Visuals**: Create a visual distortion shader or overlay for tiles within the `TimeDilationZone`.
- **Machine Speed**: Extend the time dilation to affect work speed, crafting speed, and machine output, not just movement.
- **Safety Cutoff**: Add an option to automatically disconnect the tap (losing power) if an enemy is detected.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/event_horizon_tap.rs`.
- [ ] Pops and machines within the dilation zone operate significantly slower.
- [ ] The Tap provides immense or infinite power while active.

## 7. Technical Guidance
- The `TimeDilationZone` might be better represented as a resource mapping grid coordinates if the colony gets very large, but simple distance checks are fine for the MVP.
- Ensure the `PowerSource` handles `f32::INFINITY` gracefully in the existing energy network logic to avoid NaN propagation.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
- *Builder:* How does the tap connect to Layer 2?
- *Architect:* The player constructs a `TapAnchor` in Layer 1 and an `AnomalyTether` on a station in Layer 2. If the Layer 2 station is destroyed, the connection breaks.
- *Builder:* Does time dilation affect aging/hunger?
- *Architect:* No, metabolic functions (hunger, age, rest decay) occur at normal real-time rates, creating the core tension: Pops get hungry at normal speed but walk to the cafeteria at 10% speed.
