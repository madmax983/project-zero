# 1065: Generational Linguistics

## Overview

Watching isolated colonies develop their own distinct dialects and cultural misunderstandings over time. Colonies geographically isolated or lacking communication networks slowly drift in "Linguistic Cohesion". Pops with different dialects suffer minor social penalties. Extreme drift creates entirely new languages, requiring "Translators" to conduct even basic diplomacy or trade between once-united worlds.

## Dependencies

None

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

use bevy::prelude::*;
use crate::layer1::culture::linguistics::{ColonyLanguage, DialectDrift, LingusticNetwork, translation_modifier};
use crate::layer1::entities::pop::Pop;
use crate::layer1::social::interaction::SocialInteractionEvent;

#[test]
fn test_linguistic_drift_over_time() {
    let mut app = App::new();

    // Arrange: Setup two isolated colonies with the same initial language
    let base_lang_id = 1;
    let mut colony_a = app.world_mut().spawn(ColonyLanguage { base_id: base_lang_id, drift_vector: vec![0.0, 0.0] }).id();
    let mut colony_b = app.world_mut().spawn(ColonyLanguage { base_id: base_lang_id, drift_vector: vec![0.0, 0.0] }).id();

    // Act: Advance simulation time by a significant duration without communication
    // Assume a system `apply_linguistic_drift` exists that mutates isolated `ColonyLanguage` components
    app.add_systems(Update, crate::layer1::culture::linguistics::apply_linguistic_drift);
    app.update(); // Simulate passage of time

    let lang_a = app.world().get::<ColonyLanguage>(colony_a).unwrap();
    let lang_b = app.world().get::<ColonyLanguage>(colony_b).unwrap();

    // Assert: Verify that the linguistic cohesion between the two colonies has decreased
    let drift_distance = lang_a.calculate_distance(lang_b);
    assert!(drift_distance > 0.0, "Language drift should occur over time");
}

#[test]
fn test_dialect_social_penalty() {
    // Arrange: Setup two pops with slightly drifted dialects
    let mut app = App::new();
    let pop1 = app.world_mut().spawn((Pop, DialectDrift(1.0))).id();
    let pop2 = app.world_mut().spawn((Pop, DialectDrift(3.0))).id();

    // Act: Trigger a social interaction between them
    let modifier = translation_modifier(&app.world(), pop1, pop2, None);

    // Assert: Verify a minor penalty compared to identical dialects
    assert!(modifier < 1.0 && modifier > 0.5, "Expected a minor social penalty for dialect mismatch");
}

#[test]
fn test_language_barrier_requires_translator() {
    // Arrange: Setup two pops with completely different languages (extreme drift)
    let mut app = App::new();
    let pop1 = app.world_mut().spawn((Pop, DialectDrift(1.0))).id();
    let pop2 = app.world_mut().spawn((Pop, DialectDrift(10.0))).id(); // Large drift threshold

    // Act: Attempt diplomacy or trade between them without a Translator
    let modifier = translation_modifier(&app.world(), pop1, pop2, None);

    // Assert: Verify that the action is blocked or fails completely (modifier of 0.0)
    assert_eq!(modifier, 0.0, "Communication should fail completely across a language barrier without a translator");
}

#[test]
fn test_translator_enables_communication() {
    // Arrange: Setup two pops with completely different languages and a Translator pop
    let mut app = App::new();
    let pop1 = app.world_mut().spawn((Pop, DialectDrift(1.0))).id();
    let pop2 = app.world_mut().spawn((Pop, DialectDrift(10.0))).id();
    let translator = app.world_mut().spawn((Pop, crate::layer1::culture::linguistics::Translator)).id();

    // Act: Attempt diplomacy or trade involving the Translator
    let modifier = translation_modifier(&app.world(), pop1, pop2, Some(translator));

    // Assert: Verify that the action succeeds (modifier > 0.0)
    assert!(modifier > 0.0, "Translator should enable communication across a language barrier");
}

#[test]
fn test_communication_network_prevents_drift() {
    // Arrange: Setup two colonies connected by a communication network
    let mut app = App::new();
    let base_lang_id = 1;
    let mut colony_a = app.world_mut().spawn(ColonyLanguage { base_id: base_lang_id, drift_vector: vec![0.0, 0.0] }).id();
    let mut colony_b = app.world_mut().spawn(ColonyLanguage { base_id: base_lang_id, drift_vector: vec![0.0, 0.0] }).id();

    app.world_mut().spawn(LingusticNetwork { nodes: vec![colony_a, colony_b] });

    app.add_systems(Update, crate::layer1::culture::linguistics::apply_linguistic_drift);
    app.update();

    let lang_a = app.world().get::<ColonyLanguage>(colony_a).unwrap();
    let lang_b = app.world().get::<ColonyLanguage>(colony_b).unwrap();

    // Assert: Verify that linguistic cohesion remains high and no significant drift occurs
    let drift_distance = lang_a.calculate_distance(lang_b);
    assert_eq!(drift_distance, 0.0, "Communication networks should prevent linguistic drift");
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

// Add a Language Component or Resource to track a colony's current language state.
// A simple integer or float could represent the drift from a base language.
// Implement a system that slightly mutates this state over long periods if disconnected.
// Add a check in social interaction systems to compare language states and apply penalties.
```

## REFACTOR Phase: Quality & Design

- **Performance:** Linguistic drift calculation should be infrequent (e.g., once an in-game year) rather than every tick.
- **Design:** Consider how languages are represented. A tree structure might allow for dialects to remain mutually intelligible up to a point, while completely diverging branches become distinct languages.
- **Code Smells:** Avoid hardcoding the translation penalty; use a configurable system to easily balance the impact of different levels of drift.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Linguistic drift occurs naturally in isolated populations
- [ ] Social penalties apply to interactions between drifted dialects
- [ ] Extreme drift prevents communication without a Translator
- [ ] Communication networks prevent drift

## Technical Guidance

- Use Bevy's ECS to attach a `Language` component to Pops or a `ColonyLanguage` resource to Colonies.
- The `Language` could be represented by a base ID and a drift vector or value.
- When two Pops interact, compare their `Language` states. If the difference exceeds a threshold, apply a "dialect" penalty. If it exceeds a larger threshold, treat it as a "language barrier."
- A `Translator` could be a special Job or a structure/technology that bridges the language gap, possibly by providing a temporary buff or modifying the interaction check.

## Questions

*Builder: add questions here if spec is unclear.*
