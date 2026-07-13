# 1327: The Hyperspace Wake

## 1. Overview
FTL travel leaves a scar on the universe. The faster you expand, the more you damage the fabric of space itself. Frequent hyperlane usage in a specific sector builds up "Hyperspace Wake." High wake causes sub-space anomalies, damages passing ships, and eventually tears open temporary rifts that spawn hostile entities or consume local Layer 2 infrastructure.

## 2. Dependencies
- `099` Fleet Movement
- `094` System View Architecture (for rendering lanes)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer3::hyperlane::{Hyperlane, WakeTurbulence, calculate_wake_damage};
    use crate::layer3::fleet::Fleet;

    #[test]
    fn test_wake_accumulation() {
        let mut lane = Hyperlane { wake: 0.0, ..Default::default() };
        let fleet_mass = 50.0;

        crate::layer3::hyperlane::add_wake(&mut lane, fleet_mass);

        assert!(lane.wake > 0.0);
    }

    #[test]
    fn test_wake_damage_on_transit() {
        let lane = Hyperlane { wake: 80.0, ..Default::default() }; // High wake
        let mut fleet = Fleet { health: 100.0, mass: 10.0 };

        let damage = calculate_wake_damage(&lane, &fleet);

        assert!(damage > 0.0);
    }

    #[test]
    fn test_wake_dissipation_over_time() {
        let mut lane = Hyperlane { wake: 50.0, ..Default::default() };

        crate::layer3::hyperlane::dissipate_wake(&mut lane, 1.0); // 1 tick

        assert!(lane.wake < 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation in src/layer3/hyperlane.rs
#[derive(Default)]
pub struct Hyperlane {
    pub wake: f32,
    // other fields like start_node, end_node
}

pub struct Fleet {
    pub health: f32,
    pub mass: f32,
}

pub fn add_wake(lane: &mut Hyperlane, fleet_mass: f32) {
    lane.wake += fleet_mass * 0.1;
}

pub fn calculate_wake_damage(lane: &Hyperlane, fleet: &Fleet) -> f32 {
    if lane.wake > 50.0 {
        // Damage scales with wake above safe threshold and fleet mass
        (lane.wake - 50.0) * fleet.mass * 0.01
    } else {
        0.0
    }
}

pub fn dissipate_wake(lane: &mut Hyperlane, delta_time: f32) {
    lane.wake -= 1.0 * delta_time; // Base dissipation rate
    if lane.wake < 0.0 {
        lane.wake = 0.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add a visual representation (shader or color change) to the hyperlane in the UI based on its `wake` level.
- Spawn a chronicle event if a ship takes significant damage from wake turbulence.
- Consider adding a "Rift" event if `wake` reaches 100.0, spawning an anomaly or hostile entity.

## 6. Acceptance Criteria (Testable!)
- [ ] Tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Fleets transiting a lane increase its `wake`
- [ ] High `wake` damages transiting fleets
- [ ] `wake` naturally dissipates over time

## 7. Technical Guidance
- Integrate `dissipate_wake` into the main Layer 3 simulation tick loop so all lanes cool down over time.
- Call `add_wake` immediately upon a fleet entering/exiting the lane.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
