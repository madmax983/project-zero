# 1263: The Sabotage Lottery

## 1. Overview

**Layer:** 1 / 2

**Fantasy:** Institutionalized rebellion as a pressure release valve.

**Mechanic:** When systemic unrest reaches a critical but not quite revolutionary boiling point, the colony culture might spontaneously invent the "Sabotage Lottery." Once a cycle, a random, non-critical piece of infrastructure (a streetlamp, a specific hydroponics bed, a localized decorative statue) is democratically chosen by the Pops to be destroyed. If the administration allows the destruction to stand for 5 days without repairing it, overall unrest drops significantly. If they repair it immediately, unrest spikes violently.

## 2. Dependencies

- Layer 1 Economy System (`src/layer1/economy/`)
- Layer 1 Entity System (`src/layer1/entities/`)
- Layer 1 Nature System (`src/layer1/nature/`)
- Layer 1 Culture / Unrest Tracking

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use crate::layer1::culture::sabotage_lottery::{SabotageLottery, SabotagedTarget};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_lottery_triggers_at_critical_unrest() {
        // Arrange
        let mut world = World::new();
        // Set colony unrest to just below boiling point (e.g., 90/100)

        // Act
        // Tick simulation

        // Assert
        // Verify SabotageLottery resource/event is triggered, selecting a non-critical entity
    }

    #[test]
    fn test_repairing_sabotage_too_early_spikes_unrest() {
        // Arrange
        let mut world = World::new();
        // Setup a SabotagedTarget that was destroyed recently (< 5 days)

        // Act
        // Repair the target

        // Assert
        // Verify unrest spikes violently
    }

    #[test]
    fn test_allowing_sabotage_for_duration_reduces_unrest() {
        // Arrange
        let mut world = World::new();
        // Setup a SabotagedTarget

        // Act
        // Advance time by 5 days without repairing

        // Assert
        // Verify overall unrest drops significantly
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to make tests pass
// 1. Define SabotageLottery resource/event tracking current state
// 2. Define SabotagedTarget component to track when an entity was destroyed and if it's part of the lottery
// 3. System to trigger lottery based on unrest threshold
// 4. System to handle repair attempts on SabotagedTargets
// 5. System to apply unrest reduction after 5 days of non-repair
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the lottery only selects *non-critical* infrastructure (how is "non-critical" defined? Need a trait or marker component).
- Hook into the existing building repair systems to intercept and check for `SabotagedTarget`.
- Tie the "5 days" duration to `SimulationTime`.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `sabotage_lottery.rs` or relevant modules.
- [ ] Sabotage Lottery correctly identifies and targets non-critical infrastructure.
- [ ] Repairing a Sabotage Lottery target before 5 days spikes unrest.
- [ ] Leaving a Sabotage Lottery target destroyed for 5 days reduces unrest.

## 7. Technical Guidance

- Create `src/layer1/culture/sabotage_lottery.rs`.
- Define a `SabotageLottery` state machine or resource.
- Define a `SabotagedTarget` component holding a timestamp of destruction.
- Integrate with `SimulationTime` for duration tracking.

## 8. Questions
*Builder: add questions here if spec is unclear.*
