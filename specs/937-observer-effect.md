# 937: The Observer Effect

## Overview

Quantum entanglement meets colony management. A high-tier 'Quantum Hub' generates massive amounts of power, but only functions optimally when its efficiency isn't being actively monitored by the player via the UI Inspector or when there are no Pops physically in the building. As soon as the player 'looks' at it or a Pop enters, its output collapses. Eventually, the unmonitored Hub begins generating its own bizarre, localized physics anomalies that spread through the colony.

## Dependencies

- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_quantum_hub_unmonitored_output() {
        // Arrange
        let mut app = App::new();
        // Setup Quantum Hub entity with NO observer or pops present

        // Act
        app.update();

        // Assert
        // Verify massive power generation
    }

    #[test]
    fn test_quantum_hub_output_collapses_on_observation() {
        // Arrange
        let mut app = App::new();
        // Setup Quantum Hub entity WITH an observer flag active

        // Act
        app.update();

        // Assert
        // Verify power output drops significantly or to zero
    }

    #[test]
    fn test_quantum_hub_generates_anomalies_when_unmonitored() {
        // Arrange
        let mut app = App::new();
        // Setup Quantum Hub entity unmonitored for extended duration

        // Act
        // Simulate time passing
        app.update();

        // Assert
        // Verify localized physics anomalies are spawned nearby
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests
// Create QuantumHub component and systems handling output and observation checks
```

## REFACTOR Phase: Quality & Design

- Ensure the 'observation' state is easily toggleable for UI integration.
- Structure anomaly generation so it can hook into existing event systems cleanly.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Quantum Hub output collapses correctly upon observation.
- [ ] Unmonitored hubs spawn anomalies over time.

## Technical Guidance

- You may need to create a `QuantumHub` component.
- The observation state could be represented by a resource or component flag set by the UI system when the inspector is open.

## Questions

*Builder: add questions here if spec is unclear.*
