# 1193: Heirloom Recipes

## 1. Overview
**Layer:** 1

**Fantasy:** Grandma's space-lasagna is the only thing keeping us sane.

**Mechanic:** When a Pop eats a specific combination of ingredients and has a "Great Meal" event, that recipe becomes a "Colony Tradition". Pops demand it for festivals. Providing it gives massive buffs.

**Emergence:** The colony becomes obsessed with "Rat-and-Moss Stew" because it saved them during the first winter. You have to farm rats specifically to keep morale up, even though you have high-tech nutrient paste available.

**Tension:** Efficient food (Nutrient Paste) vs. Cultural comfort (Inefficient traditional food).

## 2. Dependencies
- Base ECS system (`src/layer1/mod.rs` or relevant system)
- Layer 1 core modules

## 3. RED Phase: Tests First
```rust
// specs/1193-heirloom-recipes.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_heirloom_recipe_generation_on_great_meal() {
    let mut world = World::new();
    let pop = world.spawn(Pop).insert(ConsumedMeals { last_meal: vec![ItemType::RatMeat, ItemType::Moss] }).id();

    // Act: Run the meal processing system
    world.send_event(GreatMealEvent { pop });
    run_meal_tradition_system(&mut world);

    // Assert: Verify a "Colony Tradition" recipe is generated and saved in a global resource
    let traditions = world.resource::<ColonyTraditions>();
    assert!(traditions.recipes.contains(&vec![ItemType::RatMeat, ItemType::Moss]));
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal code to turn tests green.
// Define required components and resources.
// Update relevant systems to process the new data.
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Avoid tight coupling.
- **Performance**: Ensure systems are optimized.
- **Design**: Consider integration points.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Basic logic and edge cases are verified.

## 7. Technical Guidance
- Register all new systems, events, and resources in the main app.
- Check relevant integration points in `src/`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
