# Private Stashes

## 1. Overview
In a shortage, people look out for themselves. Pops hide resources (food, medicine) in their rooms or remote tiles. Inventory counts become inaccurate.

## 2. Dependencies
- Core architecture (Specs 001-013)
- Resource Stockpiles (Spec 022)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_anxious_pop_creates_stash() {
        // Arrange
        let mut world = World::new();
        // Pop has Anxious trait and there is a global food shortage
        world.insert_resource(GlobalResources { food: 5, ..Default::default() });
        let pop = world.spawn((Pop, Trait::Anxious, Position(0, 0), Inventory { food: 1 })).id();

        // Act
        world.run_system_once(stash_creation_system).unwrap();

        // Assert
        let stashes = world.query::<&PrivateStash>().iter(&world).count();
        assert_eq!(stashes, 1, "Anxious pop should create a stash during a shortage");
    }

    #[test]
    fn test_stash_hides_from_global_inventory() {
        // Arrange
        let mut world = World::new();
        world.spawn(PrivateStash { food: 10 });
        world.spawn(Stockpile { food: 20 });

        // Act
        let total_visible_food = calculate_visible_food(&world);

        // Assert
        assert_eq!(total_visible_food, 20, "Stashed food should not be visible to the colony");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// `stash_creation_system`: If resources are low and pop has specific traits, they remove an item from stockpile/inventory and spawn a `PrivateStash` entity.
// Global inventory calculations must ignore `PrivateStash` entities.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure pops can access their *own* stash when they are hungry.
- Add an action to "Search Rooms" that converts Stashes back into Stockpiles at a massive morale penalty.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Coverage >= 85% for new code

## 7. Technical Guidance
- Ensure `PrivateStash` has an owner field (Entity ID) so only they can use it naturally.

## 8. Questions
*Builder: add questions here if spec is unclear.*
