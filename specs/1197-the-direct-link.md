# 1197: The Direct Link

## 1. Overview
**Layer:** 1 (Meta)

**Fantasy:** "Fine, I'll do it myself." The Commander steps onto the field.

**Mechanic:** Player can "Possess" a specific Pop or Unit to take direct control (WASD movement, manual aiming). Possessed units get massive stat buffs but the global UI (Build menus, Alerts, Macro-orders) is disabled while active.

**Emergence:** You possess a sniper to make a critical shot against a raid boss. You nail it. But while zoomed in, you miss the notification that the oxygen scrubber failed. You win the fight but suffocate the colony.

**Tension:** Micro-tactical power (Heroism) vs. Macro-strategic awareness (Command).

## 2. Dependencies
- Base ECS system (`src/layer1/mod.rs` or relevant system)
- Layer 1 (Meta) core modules

## 3. RED Phase: Tests First
```rust
// specs/1197-the-direct-link.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_possess_unit_disables_global_ui() {
    let mut world = World::new();
    let pop = world.spawn(Pop).id();
    world.insert_resource(UiState { global_ui_enabled: true });

    // Act: Run the possession/UI state system
    world.send_event(PossessEntityEvent { target: pop });
    run_possession_system(&mut world);

    // Assert: Verify global UI is flagged as disabled
    let ui = world.resource::<UiState>();
    assert_eq!(ui.global_ui_enabled, false);
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
