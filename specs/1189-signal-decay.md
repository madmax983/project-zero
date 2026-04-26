# 1189: Signal Decay

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** Space is noisy. You have to shout to be heard, and sometimes you mishear the reply.

**Mechanic:** Comms messages (Trade Offers, Threats, Quests) have a "Corruption" % based on distance and interference (Nebulae/Storms). Key words are scrambled (e.g., "Demand 500 [CORRUPTED]"). Players must guess the context or boost signal power to clarify.

**Emergence:** You receive a message: "We are [CORRUPTED]! Send help!" You think it's an ally and send a rescue fleet. It was a pirate trap saying "We are boarding!"

**Tension:** Risk a guess (Speed) vs. Wait for clarity (Safety/Cost).

---

## 2. Dependencies
- Base ECS system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_signal_decay_over_distance() {
    // Arrange: Emit a signal from an origin
    let mut app = App::new();
    app.add_systems(Update, process_signal_decay);
    let origin = Vec2::ZERO;
    let target = Vec2::new(100.0, 0.0);
    let signal = app.world_mut().spawn(Signal { origin, strength: 100.0 }).id();

    // Act: Measure signal at target
    app.update();
    let measured_strength = calculate_strength_at(&app, signal, target);

    // Assert: Signal strength is lower at a distance
    assert!(measured_strength < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_signal_decay() {
    // Logic to update active signals over time if necessary
}

fn calculate_strength_at(app: &App, signal_entity: Entity, target: Vec2) -> f32 {
    let signal = app.world().get::<Signal>(signal_entity).unwrap();
    let distance = signal.origin.distance(target);
    let decay_rate = 0.5;
    (signal.strength - (distance * decay_rate)).max(0.0)
}
```

## 5. REFACTOR Phase: Quality & Design
- Refactor calculate_strength_at to pre-calculate decay fields on a grid if checking dynamically becomes too expensive.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.
- Remember to explicitly register all new systems, events, and resources to the main game app.

## 8. Questions
*Builder: add questions here if spec is unclear.*
