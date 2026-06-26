# 1311: The Dreaming Sickness

## Overview

A rare pathogen or environmental anomaly causes Pops to share a collective hallucination during sleep. This rapidly spreads, causing a new "Dream" need that must be fulfilled by sleeping, severely impacting productivity. Entire work shifts might spontaneously fall asleep to return to the dream, leaving critical infrastructure unmanned.

## Dependencies

- `004` — Pop Entity
- `005` — Pop Needs
- `034` — Pop Health

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::mind::utility_types::{PopAction, ActionType};

    #[test]
    fn test_dreaming_sickness_infection_spreads() {
        let mut app = App::new();
        // Setup systems

        let infected = app.world.spawn((Pop, DreamingSickness { severity: 0.1 })).id();
        let healthy = app.world.spawn((Pop)).id();

        // Act: Run infection spread system

        // Assert: Healthy pop should now have DreamingSickness
    }

    #[test]
    fn test_dreaming_sickness_increases_sleep_need() {
        let mut app = App::new();
        // Setup systems

        // Pop initialized with default Needs, which includes rest
        let pop = app.world.spawn((Pop, Needs::default(), DreamingSickness { severity: 0.8 })).id();

        // Act: Run sickness effect system

        // Assert: Rest need should decrease much faster than normal
    }

    #[test]
    fn test_spontaneous_sleep_at_high_severity() {
        let mut app = App::new();
        // Setup systems

        // Setup pop with idle action initially
        let pop = app.world.spawn((Pop, PopAction { current: ActionType::Idle, current_utility: 0.0, ticks_committed: 0 }, DreamingSickness { severity: 0.95 })).id();

        // Act: Run spontaneous sleep system

        // Assert: Action should change to ActionType::SatisfyRest
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct DreamingSickness {
    pub severity: f32, // 0.0 to 1.0
}

pub fn spread_dreaming_sickness(
    // Query for infected pops
    // Query for healthy pops in proximity
    // Add DreamingSickness to healthy pops based on proximity and random chance
) {
    // Implementation
}

pub fn dreaming_sickness_effects(
    // Query for pops with DreamingSickness and Needs
    // Increase rest need decay rate based on severity
) {
    // Implementation
}

pub fn spontaneous_sleep(
    // Query for pops with high severity DreamingSickness not currently sleeping
    // Force them into sleep action using PopAction
) {
    // Implementation
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Optimize the proximity check for infection spread (use spatial partitioning if available).
- **Integration**: Ensure the spontaneous sleep action correctly interrupts the current utility AI task and handles dropped items safely.
- **Design**: Hook into existing healthcare and pathogen systems introduced in 034.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Dreaming sickness spreads between pops.
- [ ] Infected pops sleep more frequently.
- [ ] High severity infections cause spontaneous sleep.

## Technical Guidance

- Integrate with the existing `Needs` system to handle the increased sleep drive (modifying the `rest` field).
- The infection spread should probably use a slow tick or a probability check rather than happening every frame.
- Consider adding a UI notification when the first pop is infected to alert the player.

## Questions

*Builder: add questions here if spec is unclear.*
