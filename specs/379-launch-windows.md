# 379: Launch Windows

## Overview

"The planets align. Space travel isn't just about distance, it's about timing."

**Launch Windows** dictates that transfer windows open based on the orbital periods of planets. The fuel cost varies wildly by date. Launching "out of window" costs 10x fuel.

This creates tension: A colony is starving, but the transfer window is closed. Do you launch now (expensive/fast) burning your entire fuel reserve to save them, leaving the fleet stranded, or wait for the window (cheap/slow)?

## Dependencies

- `099` — Fleet Movement (for navigating ships between orbital bodies)
- `117` — Fuel Consumption (for tracking and expending fuel)

## RED Phase: Tests First

Write these tests in `src/layer2/logistics/launch_window_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::logistics::launch::{TransferOrbit, LaunchCostCalculator, InWindow, OutOfWindow};
    use crate::layer2::celestial::Orbit;

    #[test]
    fn test_launch_in_window_is_cheap() {
        let mut world = World::new();

        // Planet 1 is at 0 degrees, Planet 2 is at 45 degrees
        let origin = world.spawn(Orbit { angle: 0.0, radius: 100.0 }).id();
        let destination = world.spawn(Orbit { angle: std::f32::consts::PI / 4.0, radius: 200.0 }).id();

        // Calculate transfer cost
        let cost = world.run_system_once(
            |q_orbits: Query<&Orbit>| {
                LaunchCostCalculator::calculate(origin, destination, &q_orbits)
            }
        ).unwrap();

        // Should be cheap
        assert!(matches!(cost, InWindow(fuel) if fuel == 100));
    }

    #[test]
    fn test_launch_out_of_window_is_expensive() {
        let mut world = World::new();

        // Planet 1 is at 0 degrees, Planet 2 is at 180 degrees
        let origin = world.spawn(Orbit { angle: 0.0, radius: 100.0 }).id();
        let destination = world.spawn(Orbit { angle: std::f32::consts::PI, radius: 200.0 }).id();

        let cost = world.run_system_once(
            |q_orbits: Query<&Orbit>| {
                LaunchCostCalculator::calculate(origin, destination, &q_orbits)
            }
        ).unwrap();

        // Should be 10x more expensive
        assert!(matches!(cost, OutOfWindow(fuel) if fuel == 1000));
    }
}
```

## GREEN Phase: Minimal Implementation

Implement this minimal logic in `src/layer2/logistics/launch.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer2::celestial::Orbit;

pub enum LaunchCost {
    InWindow(u32),
    OutOfWindow(u32),
}

pub struct LaunchCostCalculator;

impl LaunchCostCalculator {
    pub fn calculate(origin: Entity, destination: Entity, orbits: &Query<&Orbit>) -> Option<LaunchCost> {
        let origin_orbit = orbits.get(origin).ok()?;
        let dest_orbit = orbits.get(destination).ok()?;

        // MVP window calculation: absolute angle difference
        let angle_diff = (dest_orbit.angle - origin_orbit.angle).abs();

        // Check if within the ideal window (e.g. 45 degrees, pi/4)
        if angle_diff <= std::f32::consts::PI / 4.0 {
            Some(LaunchCost::InWindow(100))
        } else {
            // Out of window costs 10x more fuel
            Some(LaunchCost::OutOfWindow(1000))
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The MVP uses a static threshold (`std::f32::consts::PI / 4.0`). A real orbital mechanics simulation needs to take into account orbital velocity, distance/radius, and the specific impulse of the ship's engines.
- Introduce `HohmannTransfer` components to properly track the journey across the system over time.
- The UI needs to show a "Launch Window Predictor" showing the upcoming low-fuel windows to allow players to plan.

## Acceptance Criteria

- [ ] A `LaunchCostCalculator` exists to determine fuel cost between two celestial bodies.
- [ ] If the planets are aligned (in window), the fuel cost is baseline.
- [ ] If the planets are out of alignment, the fuel cost is scaled up significantly (10x).
- [ ] Test coverage for the new module is >= 85%.
- [ ] `cargo test` and `cargo clippy -- -D warnings` pass.

## Technical Guidance

- A player's fleet shouldn't be allowed to launch if they don't have enough fuel. The `LaunchCommand` should check this calculator before executing.
- `Orbit` components must correctly track their position over time via a global simulation tick.

## Questions

*Builder: add questions here if spec is unclear.*
