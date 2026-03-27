# 653: The Atrophy of Affluence

## Overview

The slow, decadent decline of a post-scarcity district where absolute ease breeds physical and mental decay. If a specific group of Pops has all their Needs met constantly (near 100% satisfaction) for an extended generational period, they develop the "Decadent" trait. Their base movement speed, resilience, and work efficiency plummet, and they develop new, increasingly bizarre, and difficult-to-fulfill "Luxury Needs" just to maintain a baseline mood. You carefully construct a fully automated utopian dome for your elite researchers. Years later, an emergency requires them to manually evacuate or repair a critical life-support system. They are too physically weak and apathetic to perform the manual labor, preferring to complain about the sudden lack of bio-acoustic ambiance while the dome slowly depressurizes around them.

## Dependencies

- None explicitly required beyond core needs and traits systems.

## RED Phase: Tests First

```rust
// src/layer1/needs_new/atrophy_of_affluence.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::pre::App;

    #[test]
    fn test_affluent_pop_develops_decadent() {
        // Arrange
        let mut app = App::new();
        // Setup a Pop with all Needs constantly met near 100%

        // Act
        // Simulate time passing

        // Assert
        // Expect the Pop to develop the "Decadent" trait after an extended period
    }

    #[test]
    fn test_decadent_trait_lowers_efficiency() {
        // Arrange

        // Act
        // A Decadent Pop attempts to work

        // Assert
        // Expect the Pop's work efficiency and movement speed to be severely reduced
    }

    #[test]
    fn test_decadent_pop_gains_luxury_needs() {
        // Arrange

        // Act
        // A Pop develops the "Decadent" trait

        // Assert
        // Expect the Pop to require increasingly bizarre and difficult "Luxury Needs" to maintain mood
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/needs_new/atrophy_of_affluence.rs

// Minimal event and system structures to make the tests pass.
// e.g., define Decadent trait, LuxuryNeeds component, etc.
```

## REFACTOR Phase: Quality & Design

- Balance the threshold for acquiring the "Decadent" trait against the difficulty of satisfying basic needs.
- Define a varied and challenging set of "Luxury Needs" that are generated when the trait is acquired.
- Consider how the presence of Decadent Pops affects the broader colony economy and social structure.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops with constantly met needs develop the "Decadent" trait over time.
- [ ] The "Decadent" trait severely reduces work efficiency, movement speed, and resilience.
- [ ] Decadent Pops require complex "Luxury Needs" to maintain their mood.

## Technical Guidance

- Use Bevy's ECS to track the historical satisfaction of a Pop's needs and apply the trait accordingly.
- The `LuxuryNeeds` component could be an extension of the existing `Needs` struct, or a separate component that interacts with the Utility AI.
- The Utility AI system will need to be updated to prioritize satisfying "Luxury Needs" for Decadent Pops.

## Questions

*Builder: add questions here if spec is unclear.*
