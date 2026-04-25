# 1163: Generational Debt

## Overview
An empire built on predatory loans from a shadowy benefactor, where the bill eventually comes due for your descendants. In the early game, an enigmatic Layer 3 entity (e.g., The Brokerage) offers massive, game-changing resource injections with no immediate downside. However, this creates "Generational Debt." Centuries later, The Brokerage returns, demanding repayment not in resources, but in specific, absurd actions: e.g., "Relocate 50% of your capital's population to a barren moon within 5 years," or "Scuttle your entire Layer 2 defensive fleet."

## Dependencies
- Layer 3 Diplomacy and Factions
- Event and Quest system for tracking long-term triggers
- Cross-layer resource injection

## RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_generational_debt_offer() {
    // Arrange: Early game state
    // Act: Trigger Brokerage offer event
    // Assert: Player receives massive resources and a GenerationalDebt component is attached to their faction
}

#[test]
fn test_generational_debt_collection_trigger() {
    // Arrange: Faction has GenerationalDebt, advance time by centuries
    // Act: Run debt collection evaluation
    // Assert: The debt collection event fires, spawning a demand quest
}

#[test]
fn test_generational_debt_failure_consequences() {
    // Arrange: Debt collection quest is active but time expires without completion
    // Act: Resolve quest failure
    // Assert: Severe negative consequences are applied (e.g. hostile fleet spawns, massive resource drain)
}
```

## GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn broker_offer_system(
    mut commands: Commands,
    mut event_reader: EventReader<TriggerBrokerOffer>,
    mut resources: Query<&mut ColonyResources>,
    mut factions: Query<(Entity, &mut Faction)>,
) {
    for event in event_reader.read() {
        // Find player faction
        for (faction_entity, mut faction) in factions.iter_mut() {
            if faction.is_player {
                // Grant massive resources
                for mut res in resources.iter_mut() {
                    res.add(ResourceType::Alloys, 50000.0);
                    res.add(ResourceType::Energy, 50000.0);
                }
                // Attach debt
                commands.entity(faction_entity).insert(GenerationalDebt {
                    cycles_until_due: 10000.0, // Long time
                });
            }
        }
    }
}

pub fn debt_collection_system(
    mut commands: Commands,
    mut factions_with_debt: Query<(Entity, &mut GenerationalDebt)>,
    time: Res<Time>,
) {
    for (entity, mut debt) in factions_with_debt.iter_mut() {
        debt.cycles_until_due -= time.delta_secs(); // Assuming delta maps to cycles for simplicity

        if debt.cycles_until_due <= 0.0 {
            // Trigger collection demand
            commands.spawn(DebtCollectionDemand {
                target_faction: entity,
                demand_type: DemandType::ScuttleFleet,
                time_limit: 100.0,
            });
            commands.entity(entity).remove::<GenerationalDebt>();
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Optimization:** Don't check the `debt_collection_system` every frame. Use a scheduled event or cron-like system for long-term timers.
- **Code Smell:** Hardcoding massive resources in the offer system. This should be configurable via an event struct or data file.
- **Integration:** The demands (like `ScuttleFleet`) need to be integrated with the quest/mission system to track player compliance.
- **Refactoring:** The `GenerationalDebt` component should track exactly what was given so the "interest" or demand severity scales appropriately.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer3/events/generational_debt.rs`
- [ ] Debt collection generates appropriate Chronicle logs for both the offer and the eventual collection.

## Technical Guidance
- **Components:** `GenerationalDebt { pub cycles_until_due: f32 }`, `DebtCollectionDemand`.
- **System Placement:** Debt tracking should be handled at the highest level (Layer 3 or meta-simulation schedule) since it spans centuries.
- **Events:** Need a robust way to verify completion of bizarre tasks like "Scuttle your entire fleet." This might require writing custom quest evaluators.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
