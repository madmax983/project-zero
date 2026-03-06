# The Black Market

## 1. Overview
Where the state fails, the underworld provides. Unmet needs spawn "Smuggler" entities at Layer 2. They dock and deliver goods but siphon credits and increase "Corruption".

## 2. Dependencies
- Core architecture (Specs 001-013)
- Trade System (Spec 039)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_unmet_needs_spawn_smugglers() {
        // Arrange
        let mut world = World::new();
        // Setup colony with massive unmet luxury needs
        world.insert_resource(ColonyStats { unmet_luxury: 100, ..Default::default() });

        // Act
        world.run_system_once(black_market_spawn_system).unwrap();

        // Assert
        let smugglers = world.query::<&Smuggler>().iter(&world).count();
        assert_eq!(smugglers, 1, "A smuggler should spawn due to unmet needs");
    }

    #[test]
    fn test_smuggler_increases_corruption() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(ColonyStats { corruption: 0.0, ..Default::default() });
        let smuggler = world.spawn(Smuggler).id();

        // Act
        world.run_system_once(smuggler_trade_system).unwrap(); // Mock trade

        // Assert
        let stats = world.get_resource::<ColonyStats>().unwrap();
        assert!(stats.corruption > 0.0, "Corruption should increase after smuggler trade");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// `black_market_spawn_system`: checks global unmet needs, has a probability to spawn a `Smuggler` entity.
// `smuggler_trade_system`: fulfills some needs but increases a `Corruption` resource/stat.
```

## 5. REFACTOR Phase: Quality & Design
- Connect `Corruption` to tax/production efficiency in existing systems.
- Ensure Smugglers use the existing `Trade` interfaces where possible.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Coverage >= 85% for new code

## 7. Technical Guidance
- Smugglers should act like regular traders but with side effects.

## 8. Questions
*Builder: add questions here if spec is unclear.*
