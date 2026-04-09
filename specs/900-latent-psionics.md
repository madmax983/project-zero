# Latent Psionics

## 1. Overview
The mind is the next frontier. The stress of the void unlocks something. Rare "Latent" trait. High Stress or exposure to "Alien Artifacts" can trigger an Awakening. Powers range from "Empathy" (Mood Aura) to "Pyrokinesis" (Fire starting) or "Foresight" (Warning of raids). Your Cook gets stressed during a famine and awakens as a Pyrokinetic. Now every time he burns the soup, he *literally* burns the kitchen. Exile the witch (Safety) or Weaponize the talent (Power)?

## 2. Dependencies
- Layer 1 Pops and Traits
- Layer 1 Stress mechanics
- Event/Chronicle system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_latent_awakening_on_high_stress() {
    let mut app = bevy::app::App::new();
    app.insert_resource(bevy::time::Time::default());

    // Arrange: Spawn a Pop with Latent trait and high Stress
    // Act: Run awakening check system
    // Assert: Verify the Pop gains an Awakened Psionic trait (e.g., Pyrokinesis)
}

#[test]
fn test_pyrokinesis_power_activation() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn an Awakened (Pyrokinetic) Pop and a Kitchen building
    // Act: Trigger an adverse event (like a failed work task)
    // Assert: Verify a fire event/entity is spawned at the Pop's location
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation for Latent trait, Awakening condition, and at least one power (e.g. Pyrokinesis)
```

## 5. REFACTOR Phase: Quality & Design
- Create an enum or trait for different Psionic powers to allow easy extension.
- Ensure awakening events are logged to the Chronicle.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement within `src/layer1/psionics.rs` or similar.

## 8. Questions
*Builder: add questions here if spec is unclear.*
