# 589: The Synthesizer's Dilemma

## 1. Overview
**Layer:** 1
**Fantasy:** Replacing nature with perfect, sterile efficiency, only to realize you miss the mess.
**Mechanic:** Late-game "Nutrient Synthesizers" produce infinite, perfectly balanced food from raw power and carbon. However, Pops eating only synthesized food slowly develop "Sensory Deprivation" stress. They crave the complex imperfections of grown food.
**Emergence:** You pave over all your farms to build synthesizers, achieving perfect food security. A year later, your colony's morale crashes. A thriving black market emerges where Pops trade rare, smuggled dirt-grown potatoes for high-value tech, forcing you to tear up a synthesizer to build a single, highly-guarded greenhouse.
**Tension:** Absolute, reliable efficiency vs. the psychological necessity of natural variety and imperfection.

## 2. Dependencies
- 008-basic-needs
- 012-farming
- 050-stress-system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_synthesized_food_consumption_increases_sensory_deprivation() {
    let mut app = setup_test_app();
    let pop = app.world.spawn((
        Pop,
        Needs::default(),
        Stress::default(),
    )).id();

    // Feed the pop exclusively synthesized food
    app.world.resource_mut::<Inventory>().add(Item::SynthesizedNutrientBlock, 10);
    app.update();

    let stress = app.world.get::<Stress>(pop).unwrap();
    assert!(stress.sensory_deprivation > 0.0);
}

#[test]
fn test_natural_food_reduces_sensory_deprivation() {
    let mut app = setup_test_app();
    let pop = app.world.spawn((
        Pop,
        Needs::default(),
        Stress { sensory_deprivation: 50.0, ..Default::default() },
    )).id();

    // Feed the pop natural food
    app.world.resource_mut::<Inventory>().add(Item::Potato, 1);
    app.update();

    let stress = app.world.get::<Stress>(pop).unwrap();
    assert!(stress.sensory_deprivation < 50.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/needs.rs
pub fn process_eating_system(
    mut query: Query<(&mut Needs, &mut Stress)>,
    mut inventory: ResMut<Inventory>,
) {
    for (mut needs, mut stress) in query.iter_mut() {
        if needs.hunger > 50.0 {
            if inventory.consume(Item::Potato, 1) {
                needs.hunger = 0.0;
                stress.sensory_deprivation = (stress.sensory_deprivation - 10.0).max(0.0);
            } else if inventory.consume(Item::SynthesizedNutrientBlock, 1) {
                needs.hunger = 0.0;
                stress.sensory_deprivation += 5.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract food types and their stress modifiers into a data-driven configuration or component traits.
- Implement a black market event triggered when `sensory_deprivation` exceeds a threshold.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Eating synthesized food increases sensory deprivation stress.
- [ ] Eating natural food decreases sensory deprivation stress.

## 7. Technical Guidance
- Update the `Stress` component to include the `sensory_deprivation` field.
- Ensure the event system handles the threshold triggers for the black market.

## 8. Questions
*Builder: add questions here if spec is unclear.*
