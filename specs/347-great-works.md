# Great Works

## 1. Overview
Leaving a mark on the universe. Multi-stage construction projects requiring massive resources and time (e.g., Space Elevator, Planetary Shield). Completion unlocks Layer 2 benefits.

## 2. Dependencies
- Core architecture (Specs 001-013)
- Construction Costs (Spec 020)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_great_work_multi_stage_construction() {
        // Arrange
        let mut world = World::new();
        let work = world.spawn((
            GreatWork::new(vec![
                Stage { resource: ResourceType::Stone, amount: 1000 },
                Stage { resource: ResourceType::Metal, amount: 500 },
            ]),
            ConstructionProgress::default()
        )).id();

        // Act - Complete stage 1
        world.run_system_once(apply_resources_to_great_work).unwrap(); // Mocked applying 1000 Stone

        // Assert
        let gw = world.get::<GreatWork>(work).unwrap();
        assert_eq!(gw.current_stage, 1, "Should advance to next stage");
        assert!(!gw.is_complete(), "Work should not be fully complete");

        // Act - Complete stage 2
        world.run_system_once(apply_resources_to_great_work).unwrap(); // Mocked applying 500 Metal

        // Assert
        let gw = world.get::<GreatWork>(work).unwrap();
        assert!(gw.is_complete(), "Work should be fully complete");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Create `GreatWork` component with stages
// Update `apply_resources` logic to handle multi-stage requirements
// Emit a `GreatWorkCompletedEvent` upon full completion
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with existing `Building` and `ConstructionProgress` systems.
- Consider UI representation of stages.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Coverage >= 85% for new code

## 7. Technical Guidance
- Avoid creating entirely new construction systems if the existing one can be adapted via an enum or stages field.

## 8. Questions
*Builder: add questions here if spec is unclear.*
