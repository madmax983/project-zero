# 650: The Fossilized Armada

## Overview

Discovering that the mountain range your colony is built on is actually the crashed remains of an ancient, colossal dreadnought. Deep Layer 1 mining operations trigger a "hull breach" event, revealing ancient, alien ship corridors instead of natural caverns. These areas contain dormant automated defenses but yield incredible technology. On Layer 2, this reveals the planet was the site of a massive historical battle, unlocking new orbital salvage sites. You greedily mine into the ship's ancient armory, inadvertently activating a long-dormant distress beacon. This summons a faction of automated warships to your system on Layer 2, turning your quiet mining colony into ground zero for a renewed ancient war.

## Dependencies

- None explicitly required beyond core map and mining systems.

## RED Phase: Tests First

```rust
// src/layer1/events_new/fossilized_armada.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::pre::App;

    #[test]
    fn test_mining_triggers_hull_breach() {
        // Arrange
        let mut app = App::new();
        // Setup initial world state: add mining operation deep in the crust

        // Act
        // Trigger mining completion

        // Assert
        // Expect a `HullBreachEvent` to be fired, revealing an ancient corridor
    }

    #[test]
    fn test_hull_breach_spawns_defenses() {
        // Arrange

        // Act
        // Trigger a hull breach

        // Assert
        // Expect dormant automated defense entities to be spawned in the new corridor
    }

    #[test]
    fn test_mining_armory_activates_beacon() {
        // Arrange

        // Act
        // Mine an ancient armory tile

        // Assert
        // Expect a distress beacon to be activated and Layer 2 automated warships summoned
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/events_new/fossilized_armada.rs

// Minimal event and system structures to make the tests pass.
// e.g., define HullBreachEvent, AutomatedDefense component, etc.
```

## REFACTOR Phase: Quality & Design

- Ensure the probability of triggering a hull breach is balanced against the depth of mining.
- Modularize the generation of ancient ship corridors to allow for varied layouts and loot tables.
- Consider how the presence of automated defenses interacts with the existing combat and utility AI systems.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Mining deep in specific "mountain" regions has a chance to trigger a hull breach.
- [ ] Hull breaches spawn ancient corridors with technology and defenses.
- [ ] Mining specific ancient armory tiles activates a distress beacon, summoning Layer 2 fleets.

## Technical Guidance

- Use Bevy events to decouple the mining action from the hull breach generation.
- The ancient ship corridors could be a distinct terrain type or a separate map layer overlaid on the existing cavern system.
- The Layer 2 fleet summoning will require integration with the system-layer simulation.

## Questions

*Builder: add questions here if spec is unclear.*
