# 1101 - Bureaucratic Ghost Towns

## 1. Overview
The "Bureaucratic Ghost Towns" feature simulates a scenario where a Layer 1 colony has completely died out (e.g., due to starvation or disaster), but the automated bureaucratic systems of the Layer 3 Empire AI continue to function, unaware of the actual ground truth. The dead colony continues to receive resources, demands taxes, and accumulates wealth, becoming a silent, automated fortress.

## 2. Dependencies
- Layer 1 core simulation (Colony state, Population counts)
- Layer 3 abstraction (Empire AI, Resource shipments, Tax demands)
- Interaction layer between Layer 1 and Layer 3 (Cross-layer communication/reporting)

## 3. RED Phase: Tests First

```rust
// tests/integration/bureaucratic_ghost_towns.rs

#[test]
fn test_ghost_town_continues_receiving_shipments() {
    // Arrange: Create a colony with zero population but an active automated reporting system.
    // Act: Advance the Layer 3 simulation tick where resources are distributed.
    // Assert: The ghost town's resource stockpile increases, despite having no living Pops.
}

#[test]
fn test_ghost_town_maintains_defenses() {
    // Arrange: Create a dead colony with automated defenses.
    // Act: Trigger an invasion or looting event against the colony.
    // Assert: The automated defenses successfully engage the attackers.
}

#[test]
fn test_discovery_of_ghost_town() {
    // Arrange: Create a dead colony that is receiving resources.
    // Act: Send an expedition or scout from another colony/faction to the ghost town.
    // Assert: The scout discovers the vast stockpiles and the lack of living population, triggering a "Ghost Town Discovered" event.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/bureaucracy.rs

#[derive(Component)]
pub struct AutomatedReporting {
    pub is_active: bool,
    pub reported_population: usize, // The fake number sent to the capital
}

#[derive(Component)]
pub struct AutomatedDefenses {
    pub power_level: f32,
    pub is_active: bool,
}

// System to simulate the delayed realization of the colony's death
pub fn empire_resource_distribution_system(
    mut commands: Commands,
    mut colonies: Query<(Entity, &mut ResourceStockpile, &AutomatedReporting)>,
) {
    for (entity, mut stockpile, reporting) in colonies.iter_mut() {
        if reporting.is_active && reporting.reported_population > 0 {
            // The empire thinks the colony is alive and sends resources
            stockpile.food += 100;
            stockpile.credits += 500;
        }
    }
}

// System where Layer 1 updates the reporting (or fails to if everyone is dead but automation is on)
pub fn colony_reporting_system(
    mut colonies: Query<(&Population, &mut AutomatedReporting)>,
) {
    for (pop, mut reporting) in colonies.iter_mut() {
        if pop.count == 0 {
            // Automation might continue reporting the last known good number if not explicitly shut down
            // For MVP, we'll just leave reported_population as it was before they died.
        } else {
            reporting.reported_population = pop.count;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Cross-Layer Sync**: Ensure that the `AutomatedReporting` component gracefully handles the eventual realization that the colony is dead (e.g., after X ticks of no actual activity, or when an auditor arrives).
- **Defensive Behavior**: The `AutomatedDefenses` should integrate with the existing combat/invasion resolution systems.
- **Looting Mechanic**: Implement the logic for *how* a player or AI faction can siege the ghost town to claim its hoarded resources.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new bureaucratic systems.
- [ ] A colony with 0 Pops but active `AutomatedReporting` continues to accrue resources from Layer 3.
- [ ] Automated defenses remain functional even with 0 Pops.

## 7. Technical Guidance
- **Reporting Disconnect**: The core mechanic relies on a disconnect between the *actual* state of the colony (Layer 1) and the *reported* state (Layer 3). Maintain this separation clearly.
- **Resource Hoarding**: Ensure the colony's storage capacity can handle the continuous influx of resources, or implement a mechanic where excess resources are "vented" or wasted if storage is full.

## 8. Questions
*Builder: add questions here if spec is unclear.*
