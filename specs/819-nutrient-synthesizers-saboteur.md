# The Nutrient Synthesizer's Saboteur (Spec 819)

## 1. Overview
Advanced Nutrient Synthesizers produce food efficiently, but a disgruntled Pop with high stress can perform an act of sabotage: "Flavor Imprinting." They secretly encode a specific emotional trauma (e.g., despair, terror) into the synthesizer's flavor profile matrix. Every Pop who eats the food experiences a sudden, inexplicable shared emotional breakdown.

## 2. Dependencies
- Layer 1 core systems (Pops, Food/Nutrient Synthesizers, Stress, Morale)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_nutrient_synthesizers_saboteur_basic_behavior() {
    // Arrange
    let mut app = App::new();

    // Setup a nutrient synthesizer
    let synth = app.world_mut().spawn((Building, NutrientSynthesizer)).id();

    // Setup a highly stressed pop
    let saboteur = app.world_mut().spawn((Pop, Stress(0.9))).id();

    // Act
    // Simulate sabotage action
    app.world_mut().send_event(SabotageEvent { pop: saboteur, target: synth, sabotage_type: SabotageType::FlavorImprinting });
    app.update();

    // Assert
    // Synthesizer should have FlavorImprint component
    assert!(app.world().entity(synth).contains::<FlavorImprint>());

    // Setup another pop that eats from the synthesizer
    let consumer = app.world_mut().spawn((Pop, Stress(0.1), Morale(0.8))).id();
    app.world_mut().send_event(EatEvent { pop: consumer, source: synth });
    app.update();

    // Assert
    // Consumer should experience sudden stress increase and morale drop
    assert!(app.world().get::<Stress>(consumer).unwrap().0 > 0.1);
    assert!(app.world().get::<Morale>(consumer).unwrap().0 < 0.8);
}

#[test]
fn test_nutrient_synthesizers_saboteur_edge_cases() {
    // Arrange
    let mut app = App::new();

    // Setup a nutrient synthesizer with an existing imprint
    let synth = app.world_mut().spawn((Building, NutrientSynthesizer, FlavorImprint)).id();

    // Setup a pop with max stress trying to sabotage again
    let saboteur = app.world_mut().spawn((Pop, Stress(1.0))).id();

    // Act
    app.world_mut().send_event(SabotageEvent { pop: saboteur, target: synth, sabotage_type: SabotageType::FlavorImprinting });
    app.update();

    // Assert
    // Imprint should not stack uncontrollably or cause errors
    assert!(app.world().entity(synth).contains::<FlavorImprint>());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Create a generic `Sabotageable` trait or component that can be applied to other critical infrastructure.
- Balance the threshold at which highly stressed pops decide to sabotage.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Listen for `SabotageEvent` to attach `FlavorImprint` to `NutrientSynthesizer` entities.
- Modify the `EatEvent` handling logic: if the food source has `FlavorImprint`, apply stress/morale penalties to the consumer.

## 8. Questions
*Builder: add questions here if spec is unclear.*
