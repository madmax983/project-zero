# 566: The Synthesizer's Dilemma

## Overview

Replacing nature with perfect, sterile efficiency, only to realize you miss the mess. Late-game "Nutrient Synthesizers" produce infinite, perfectly balanced food from raw power and carbon. However, Pops eating only synthesized food slowly develop "Sensory Deprivation" stress. They crave the complex imperfections of grown food.

## Dependencies

- None explicitly, but relies on `PopNeeds` and `StressSystem` (Layer 1).


- `NutrientSynthesizer` building that produces `SyntheticFood`.
- Pops consuming `SyntheticFood` gain a hidden `SensoryDeprivation` stress accumulator.
- High `SensoryDeprivation` leads to a new stress breakdown type: `CravingImperfection`.
- Eating `NaturalFood` (grown/farmed) rapidly reduces `SensoryDeprivation`.

- Completely disabling `SyntheticFood` consumption; they will still eat it, they just get stressed.

## RED Phase: Tests First

```rust
// tests/integration/synthesizer_dilemma.rs

#[test]
fn test_eating_synthetic_food_increases_sensory_deprivation() {
    let mut app = setup_test_app();
    let pop = spawn_test_pop(&mut app);

    // Feed pop synthetic food
    feed_pop(&mut app, pop, FoodType::Synthetic);
    app.update();

    // Assert sensory deprivation increases
    let stress = app.world().get::<Stress>(pop).unwrap();
    assert!(stress.sensory_deprivation > 0.0);
}

#[test]
fn test_eating_natural_food_clears_sensory_deprivation() {
    let mut app = setup_test_app();
    let pop = spawn_test_pop_with_sensory_deprivation(&mut app, 50.0);

    // Feed pop natural food
    feed_pop(&mut app, pop, FoodType::Natural);
    app.update();

    // Assert sensory deprivation is cleared or significantly reduced
    let stress = app.world().get::<Stress>(pop).unwrap();
    assert!(stress.sensory_deprivation < 50.0);
}

#[test]
fn test_high_sensory_deprivation_triggers_craving_breakdown() {
    let mut app = setup_test_app();
    let pop = spawn_test_pop_with_sensory_deprivation(&mut app, 100.0); // Threshold

    app.update();

    // Assert pop has a breakdown of type CravingImperfection
    let breakdown = app.world().get::<MentalBreakdown>(pop).unwrap();
    assert_eq!(breakdown.break_type, BreakType::CravingImperfection);
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/food.rs
#[derive(Component)]
pub struct SensoryDeprivation(pub f32);

// src/layer1/systems/food_systems.rs
pub fn process_eating_system(
    mut query: Query<(&mut SensoryDeprivation, &mut Hunger, &Diet)>,
    mut events: EventReader<EatEvent>,
) {
    for event in events.read() {
        if let Ok((mut dep, mut hunger, diet)) = query.get_mut(event.pop) {
            match event.food_type {
                FoodType::Synthetic => {
                    dep.0 += 5.0; // Increase deprivation
                }
                FoodType::Natural => {
                    dep.0 = (dep.0 - 20.0).max(0.0); // Decrease deprivation
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactoring:** Extract the food type logic into a configurable data-driven approach rather than hardcoding `5.0` and `20.0` in the system.
- **Code Smells:** Avoid `match` statements on `FoodType` everywhere; instead, have `FoodType` define its nutritional and psychological stats.
- **Performance:** Ensure we aren't iterating over all pops every tick, only those who ate.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops eating only synthetic food eventually suffer a breakdown.
- [ ] Eating natural food relieves the condition.

## Technical Guidance

### Components
```rust
#[derive(Component)]
pub struct SensoryDeprivation(pub f32);

pub enum FoodType {
    Synthetic,
    Natural,
}
```

### Systems
```rust
pub fn apply_sensory_deprivation_stress_system(...) {}
pub fn check_craving_breakdown_system(...) {}
```

### Integration Points
Connect this to the `StressSystem`. When a pop has high `SensoryDeprivation`, it should contribute to their overall `Stress` component.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
