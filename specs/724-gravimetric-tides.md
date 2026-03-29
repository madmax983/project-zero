# 724 The Gravimetric Tides

## 1. Overview
The colony sits at the mercy of celestial mechanics. Massive planetary bodies passing close to the colony's world periodically alter the local gravity for extended durations. High tides increase hauling and building speed but drastically increase the risk of structural collapse and workplace injuries. Low tides provide immense stability but make movement sluggish and exhaustive.

## 2. Dependencies
- 013 Schedule and System Execution Ordering
- 017 Designation System
- 025 Hauling Logistics
- 035 Workplace Hazards

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gravimetric_tide_high_increases_speed_and_risk() {
        let mut world = World::new();
        // Setup initial world state
        world.insert_resource(GravimetricTideState::High);

        // Spawn a pop with hauling task
        let pop = world.spawn((Pop, MovementSpeed(1.0), InjuryRisk(0.01))).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_gravimetric_tides);
        schedule.run(&mut world);

        let speed = world.get::<MovementSpeed>(pop).unwrap();
        let risk = world.get::<InjuryRisk>(pop).unwrap();

        assert!(speed.0 > 1.0, "High tide should increase movement speed");
        assert!(risk.0 > 0.01, "High tide should increase injury risk");
    }

    #[test]
    fn test_gravimetric_tide_low_decreases_speed_and_risk() {
        let mut world = World::new();
        world.insert_resource(GravimetricTideState::Low);

        let pop = world.spawn((Pop, MovementSpeed(1.0), InjuryRisk(0.01))).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_gravimetric_tides);
        schedule.run(&mut world);

        let speed = world.get::<MovementSpeed>(pop).unwrap();
        let risk = world.get::<InjuryRisk>(pop).unwrap();

        assert!(speed.0 < 1.0, "Low tide should decrease movement speed");
        assert!(risk.0 < 0.01, "Low tide should decrease injury risk");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default, PartialEq)]
pub enum GravimetricTideState {
    #[default]
    Normal,
    High,
    Low,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct InjuryRisk(pub f32);

pub fn apply_gravimetric_tides(
    tide_state: Res<GravimetricTideState>,
    mut query: Query<(&mut MovementSpeed, &mut InjuryRisk), With<Pop>>,
) {
    for (mut speed, mut risk) in query.iter_mut() {
        match *tide_state {
            GravimetricTideState::High => {
                speed.0 *= 1.5;
                risk.0 *= 2.0;
            }
            GravimetricTideState::Low => {
                speed.0 *= 0.5;
                risk.0 *= 0.5;
            }
            GravimetricTideState::Normal => {}
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently multipliers are hardcoded. We should move them into a `GravimetricTideConfig` resource.
- Need to ensure we don't apply multipliers multiple times each tick; might need a base value component.

## 6. Acceptance Criteria
- [ ] `apply_gravimetric_tides` modifies movement speed and injury risk appropriately depending on the `GravimetricTideState`.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- Implement a base speed and risk component to avoid compounding multipliers continuously over multiple ticks.
- Ensure the state transitions smoothly between tides.

## 8. Questions
*Builder: add questions here if spec is unclear.*
