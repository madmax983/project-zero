# 1276: Signal Decay

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** Space is noisy. You have to shout to be heard, and sometimes you mishear the reply.

**Mechanic:** Comms messages (Trade Offers, Threats, Quests) have a "Corruption" % based on distance and interference (Nebulae/Storms). Key words are scrambled (e.g., "Demand 500 [CORRUPTED]"). Players must guess the context or boost signal power to clarify.

## 2. Dependencies
- Base simulation framework (`App`, `World`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_signal_decay_basic_behavior() {
        // Arrange
        let mut app = App::new();
        // Act
        // Assert
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components and systems here
```

## 5. REFACTOR Phase: Quality & Design
- TBD

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Implement MVP

## 8. Questions
*Builder: add questions here if spec is unclear.*
