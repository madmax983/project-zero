# 938: The Diplomatic Hostage

## Overview

Playing high-stakes politics with people's lives. When negotiating peace treaties with rival empires, you can offer or demand 'Ward' Pops. A Ward is a high-value Pop (like the governor's child) sent to live in the rival empire. While the Ward is safe, the peace treaty holds. If the Ward is mistreated or killed, war immediately resumes with massive Casus Belli bonuses.

## Dependencies

- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ward_exchange_establishes_peace() {
        // Arrange
        let mut app = App::new();
        // Setup two factions in a state of war

        // Act
        // Execute a ward exchange
        app.update();

        // Assert
        // Verify the factions are now at peace and the ward is relocated
    }

    #[test]
    fn test_ward_mistreatment_triggers_war() {
        // Arrange
        let mut app = App::new();
        // Setup a peace treaty maintained by a ward

        // Act
        // Apply negative modifiers/damage to the ward pop
        app.update();

        // Assert
        // Verify war resumes with Casus Belli bonuses
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests
// Define Ward component and systems handling diplomacy states
```

## REFACTOR Phase: Quality & Design

- Ensure diplomacy state transitions fire the correct events for UI and logging.
- Generalize the concept of "mistreatment" to hook into existing Pop health/morale systems.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Exchanging a ward enforces a peace state.
- [ ] Mistreating a ward successfully breaks the peace and gives Casus Belli.

## Technical Guidance

- A `Ward` component could track the home faction and host faction.
- Check the integration points with Layer 3 diplomacy mechanics if they exist.

## Questions

*Builder: add questions here if spec is unclear.*
