# 855 - Spare Parts Economy

## 1. Overview
High-tech buildings require "Precursor Parts" (e.g., Quantum Cores) that cannot be crafted, only scavenged or traded. Maintaining these buildings consumes these parts, forcing players to explore or trade to sustain advanced infrastructure.

## 2. Dependencies
- `src/layer1/inventory.rs` or resource system
- `src/layer1/infrastructure/` (Building maintenance)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal systems
        app
    }

    #[test]
    fn test_high_tech_building_consumes_precursor_parts() {
        let mut app = setup_app();
        // Arrange: Teleporter building and Colony Inventory with Precursor Parts
        // Act: Run maintenance tick
        // Assert: Precursor Part consumed from inventory
    }

    #[test]
    fn test_high_tech_building_shuts_down_without_parts() {
        let mut app = setup_app();
        // Arrange: Teleporter building, empty Colony Inventory
        // Act: Run maintenance tick
        // Assert: Building state becomes Inoperable/Shutdown
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct RequiresPrecursorParts {
    pub part_type: String,
    pub amount_per_tick: u32,
}

pub fn precursor_maintenance_system(
    mut buildings: Query<(&RequiresPrecursorParts, &mut BuildingState)>,
    // mut inventory: ResMut<ColonyInventory>,
) {
    // Attempt to consume parts from inventory.
    // If successful, state is Active.
    // If not, state is Inoperable.
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an event `PrecursorShortageEvent` when a building shuts down to inform the player.
- Ensure the item/resource type is correctly registered in the global enum or registry.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Buildings consume uncraftable precursor parts over time.
- [ ] Buildings shut down when parts run out.

## 7. Technical Guidance
- You will need to add a new uncraftable item type (e.g., `ItemType::QuantumCore`) to the resource definitions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
