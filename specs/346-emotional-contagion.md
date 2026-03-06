# Emotional Contagion

## 1. Overview
Panic spreads like wildfire. Laughter is infectious. The mood of the crowd overpowers the individual. This feature implements extreme emotions (Terror, Joy, Rage) having a radius. Pops interacting with or near emotional pops receive a temporary mood modifier.

## 2. Dependencies
- Core architecture (Specs 001-013)
- Pop Needs (Spec 005)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::Trait;

    #[test]
    fn test_emotional_contagion_spreads_joy() {
        // Arrange
        let mut world = World::new();
        // Setup grid and pops...
        let happy_pop = world.spawn((Pop, Needs { leisure: 1.0, ..Needs::default() }, Position(0, 0))).id();
        let neutral_pop = world.spawn((Pop, Needs { leisure: 0.5, ..Needs::default() }, Position(1, 0))).id();

        // Act
        world.run_system_once(emotional_contagion_system).unwrap();

        // Assert
        let needs = world.get::<Needs>(neutral_pop).unwrap();
        assert!(needs.leisure > 0.5, "Neutral pop should gain leisure from nearby happy pop");
    }

    #[test]
    fn test_emotional_contagion_spreads_terror() {
        // Arrange
        let mut world = World::new();
        let terrified_pop = world.spawn((Pop, Needs { rest: 0.1, leisure: 0.1, ..Needs::default() }, Position(0, 0))).id();
        let neutral_pop = world.spawn((Pop, Needs { rest: 0.8, leisure: 0.8, ..Needs::default() }, Position(1, 0))).id();

        // Act
        world.run_system_once(emotional_contagion_system).unwrap();

        // Assert
        let needs = world.get::<Needs>(neutral_pop).unwrap();
        assert!(needs.leisure < 0.8, "Neutral pop should lose leisure from nearby terrified pop");
    }

    #[test]
    fn test_emotional_contagion_range_limit() {
        // Arrange
        let mut world = World::new();
        let happy_pop = world.spawn((Pop, Needs { leisure: 1.0, ..Needs::default() }, Position(0, 0))).id();
        let far_pop = world.spawn((Pop, Needs { leisure: 0.5, ..Needs::default() }, Position(10, 10))).id();

        // Act
        world.run_system_once(emotional_contagion_system).unwrap();

        // Assert
        let needs = world.get::<Needs>(far_pop).unwrap();
        assert_eq!(needs.leisure, 0.5, "Far pop should be unaffected");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Implement `emotional_contagion_system`
// - Query all Pops with their Positions and Needs
// - For each Pop, if their Needs.morale() is very high (>0.8) or very low (<0.2):
//     - Iterate over other Pops within a short radius (e.g. 2 tiles)
//     - Apply a small positive/negative modifier to their leisure need
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the spatial query (don't compare every pop to every other pop if N is large; use spatial partitioning if available, or just restrict it to a small radius check).
- Ensure modifiers don't stack infinitely or ping-pong.
- Add components like `TemporaryMoodModifier` to handle the buff/debuff duration instead of instantly changing Needs.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Coverage >= 85% for new code
- [ ] Joy and Terror spread to nearby pops correctly

## 7. Technical Guidance
- Be careful with `O(N^2)` distance checks. Use the `TerrainGrid` or a similar spatial index if Pop count is high.
- A new component like `RecentContagion` could prevent the same pop from being affected multiple times per tick.

## 8. Questions
*Builder: add questions here if spec is unclear.*
