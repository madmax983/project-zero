# 849 - Generational Wealth

## 1. Overview
The original colonists become an entrenched aristocracy, hoarding resources while new arrivals suffer. Pops can pass down their personal inventory (tools, high-quality clothes, saved rations) to their descendants. Over generations, "Founder Families" accumulate massive personal stockpiles, creating inequality and tension when crises hit.

## 2. Dependencies
- `src/layer1/inventory.rs` (Pop personal inventory)
- `src/layer1/genetics.rs` or family/descendant tracking
- `src/layer1/needs.rs` (Hunger/Ration consumption)
- `src/layer1/social.rs` (Inequality tension)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::inventory::Inventory;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal systems
        app
    }

    #[test]
    fn test_wealth_inheritance_on_death() {
        let mut app = setup_app();
        // Arrange: Parent pop with high wealth and a child pop
        // Act: Parent dies
        // Assert: Child's inventory increases by parent's inventory
    }

    #[test]
    fn test_founder_family_accumulation() {
        let mut app = setup_app();
        // Arrange: Simulate multiple generations
        // Act: Pass wealth down
        // Assert: Latest generation has significantly higher wealth than new arrivals
    }

    #[test]
    fn test_inequality_tension_generation() {
        let mut app = setup_app();
        // Arrange: High wealth pop adjacent to starving low wealth pop
        // Act: Run tension system
        // Assert: Tension/Unrest increases for the low wealth pop
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Systems
pub fn inheritance_system(
    mut death_events: EventReader<crate::layer1::events::PopDeathEvent>,
    mut inventories: Query<&mut crate::layer1::inventory::Inventory>,
    family_tree: Res<crate::layer1::genetics::FamilyTree>,
) {
    for event in death_events.read() {
        if let Some(descendant) = family_tree.get_heir(event.entity) {
            // Transfer items
            // let parent_inv = inventories.get(event.entity).unwrap().clone();
            // let mut heir_inv = inventories.get_mut(descendant).unwrap();
            // heir_inv.merge(parent_inv);
        }
    }
}

pub fn inequality_tension_system(
    mut pops: Query<(&crate::layer1::inventory::Inventory, &mut crate::layer1::needs::Needs)>,
) {
    // Compare wealth and adjust morale/tension
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a dedicated `InheritanceEvent` to decouple death logic from inventory transfer.
- Consider adding a "Wealth" metric to `Inventory` to easily compare pops instead of counting individual items every frame.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Wealth transfers from parent to child on death.
- [ ] Inequality between wealthy and poor pops generates unrest/tension.

## 7. Technical Guidance
- Hook into the existing death systems. Make sure items aren't dropped on the ground if they are inherited.
- Tension should be calculated periodically or based on events (like a famine), not every frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
