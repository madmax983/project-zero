# 961: The Silent World

## 1. Overview
A planet type with exactly zero native fauna and an incredibly fast-growing, highly nutritious single species of flora. The flora is perfectly safe to eat and provides massive growth bonuses, but emits a microscopic airborne compound that completely nullifies all sound within a 5-tile radius on Layer 1, causing noise-based alerts to fail.

## 2. Dependencies
- `010` Pops
- `045` Flora & Fauna
- `080` Alert Systems

## 3. RED Phase: Tests First
```rust
#[test]
fn test_silent_flora_growth_bonus() {
    // Arrange: A Pop consuming the Silent Flora.
    let mut app = App::new();

    // Act: Advance time.
    app.update();

    // Assert: Pop growth/health bonuses are correctly applied from high nutrition.
}

#[test]
fn test_silent_flora_nullifies_sound() {
    // Arrange: A building emitting a noise alert within 5 tiles of Silent Flora.
    let mut app = App::new();

    // Act: Try to trigger or propagate the sound alert.
    app.update();

    // Assert: The sound alert fails to propagate or trigger properly due to the flora.
}

#[test]
fn test_silent_flora_no_native_fauna() {
    // Arrange: Generating a Silent World planet.
    let mut app = App::new();

    // Act: Spawn planet entities.
    app.update();

    // Assert: Zero native fauna are spawned.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add a specific planet trait `SilentWorld` that disables fauna generation in the map generator.
// Introduce `SilentFlora` component giving large nutritional stats.
// Modify `alert_system` (or sound propagation logic) to drop/ignore audio events that originate or travel within 5 tiles of an entity with `SilentFlora`.
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the 5-tile radius check using spatial hashing or a grid lookup instead of checking every tile every frame for sound events.
- Add UI visual cues (like a muted icon) over buildings suffering critical failures near silent flora so players can still visually track issues.
- Introduce an event `SilentFloraDiscoveredEvent` to trigger a lore/chronicle entry.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Silent Flora provides a high growth/nutrition bonus to Pops.
- [ ] Auditory alerts are demonstrably blocked within a 5-tile radius of the flora.
- [ ] Silent Worlds generate with zero fauna.

## 7. Technical Guidance
- Integrate with `src/layer1/nature/flora.rs` and `src/layer1/systems/alerts.rs`.
- The sound nullification should only block gameplay audio alerts or logical sound events for Pop AI, not necessarily the actual game engine audio depending on the implementation pattern.

## 8. Questions
*Builder: add questions here if spec is unclear.*
