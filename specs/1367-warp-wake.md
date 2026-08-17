# 1367: Warp Wake

## 1. Overview
**Layer:** 2

**Fantasy:** Traffic in space. The faster they go, the harder it is for you.

**Mechanic:** FTL ships leave a "Wake" of distorted space that slows down other ships traveling the same lane for a duration. High traffic lanes become "Muddy", forcing slower travel or alternate routes.

**Emergence:** The trade route to the Hub is so busy it's actually faster to fly the "Long Way" around the nebula than to slog through the wake of a thousand freighters.

**Tension:** Centralized highways (congestion) vs. Distributed paths (inefficiency).

## 2. Dependencies
- Base ECS system
- Layer 2 Fleet movement system
- Hyperlane navigation system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{Fleet, MovementSpeed};
    use crate::layer2::navigation::Hyperlane;
    use crate::simulation::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime { tick: 0 });
        app.add_systems(Update, (
            apply_warp_wake_system,
            decay_warp_wake_system,
        ));
        app
    }

    #[test]
    fn test_warp_wake_slows_down_fleets() {
        let mut app = setup_app();

        // Setup hyperlane with wake
        let hyperlane = app.world_mut().spawn(Hyperlane {
            warp_wake_intensity: 5.0, // High intensity wake
        }).id();

        // Spawn a fleet on the lane
        let fleet = app.world_mut().spawn((
            Fleet,
            MovementSpeed { base: 10.0, current: 10.0 },
            CurrentHyperlane { lane: hyperlane },
        )).id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(fleet).unwrap();

        // Speed should be reduced by wake
        assert!(speed.current < speed.base);
    }

    #[test]
    fn test_warp_wake_decays_over_time() {
        let mut app = setup_app();

        // Setup hyperlane with wake
        let hyperlane = app.world_mut().spawn(Hyperlane {
            warp_wake_intensity: 5.0,
        }).id();

        app.world_mut().resource_mut::<SimulationTime>().tick += 100;
        app.update();

        let lane = app.world().get::<Hyperlane>(hyperlane).unwrap();

        // Wake intensity should have decreased
        assert!(lane.warp_wake_intensity < 5.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::simulation::SimulationTime;

#[derive(Component)]
pub struct Hyperlane {
    pub warp_wake_intensity: f32,
}

#[derive(Component)]
pub struct MovementSpeed {
    pub base: f32,
    pub current: f32,
}

#[derive(Component)]
pub struct CurrentHyperlane {
    pub lane: Entity,
}

pub fn apply_warp_wake_system(
    mut fleet_query: Query<(&mut MovementSpeed, &CurrentHyperlane)>,
    lane_query: Query<&Hyperlane>,
) {
    for (mut speed, current_lane) in fleet_query.iter_mut() {
        if let Ok(lane) = lane_query.get(current_lane.lane) {
            // Reduce speed based on wake intensity (e.g., 10% reduction per intensity point, max 90% reduction)
            let reduction_factor = 1.0 - (lane.warp_wake_intensity * 0.1).min(0.9);
            speed.current = speed.base * reduction_factor;
        }
    }
}

pub fn decay_warp_wake_system(
    mut lane_query: Query<&mut Hyperlane>,
) {
    // Simple decay: reduce wake intensity by 0.1 per tick
    for mut lane in lane_query.iter_mut() {
        if lane.warp_wake_intensity > 0.0 {
            lane.warp_wake_intensity = (lane.warp_wake_intensity - 0.1).max(0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Allow player to research "Advanced Warp Drives" that mitigate wake effects or generate less wake.
- Implement a `GenerateWarpWake` system that triggers when a fleet successfully completes a jump on a hyperlane.
- Update the UI to visually display "muddy" hyperlanes (e.g. changing color or thickness based on intensity).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Fleets traveling on a `Hyperlane` with `warp_wake_intensity` > 0 have their `MovementSpeed.current` reduced.
- [ ] `Hyperlane.warp_wake_intensity` decays gradually over time.

## 7. Technical Guidance
- Hook into the existing movement resolution system for fleets.
- Ensure the decay rate is balanced against typical fleet travel frequencies to prevent permanent lockdown of major routes.

## 8. Questions
*Builder: add questions here if spec is unclear.*
