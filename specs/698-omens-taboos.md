# 698: Omens & Taboos

## 1. Overview
**Layer:** 1
**Fantasy:** In the face of the unknown, people find patterns. "Don't dig on Tuesdays."
**Mechanic:** When a negative event occurs (Raid, Collapse), the game logs recent actions (e.g., "Mining Iron"). That action gains a temporary "Taboo" status. Pops gain Stress/Fear when performing Taboo actions.
**Emergence:** Production of a vital resource halts because a miner broke his leg, and now the whole colony thinks the Iron vein is cursed.
**Tension:** Force the work (High Stress) or respect the fear (Resource shortage).

## 2. Dependencies
- `009` Job System
- `034` Pop Health (Stress)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_negative_event_creates_taboo_on_recent_action() {
    // Arrange: A negative event occurs (e.g., Collapse), recent actions logged
    // Act: Process omen/taboo system
    // Assert: A Taboo is created for the most prominent recent action (e.g., Mining)
}

#[test]
fn test_performing_taboo_action_increases_stress() {
    // Arrange: A Pop performing an action that is currently marked as Taboo
    // Act: Process action effects
    // Assert: Pop's stress increases
}

#[test]
fn test_taboos_expire_over_time() {
    // Arrange: A temporary Taboo exists
    // Act: Advance simulation time past Taboo duration
    // Assert: Taboo is removed
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct Taboo { action_type: ActionType, duration: f32 }
// fn evaluate_taboos_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- Review `UtilityWeights` for pop trait/needs scaling.
- Ensure event broadcasts don't cause performance issues when scaling.
- Eliminate duplicate spatial queries where possible.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement custom event/component in `layer1` as per standard architectural practices.
- Consider utilizing existing ECS query filters instead of creating new marker structs unnecessarily.
- Adhere strictly to RED-GREEN-REFACTOR.

## 8. Questions
*Builder: add questions here if spec is unclear.*
