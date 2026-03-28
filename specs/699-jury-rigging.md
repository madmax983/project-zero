# 699: Jury-Rigging

## 1. Overview
**Layer:** 1
**Fantasy:** The MacGyver solution. Keeping the station running with duct tape and prayers.
**Mechanic:** Broken/Damaged buildings can be "Jury-Rigged" instantly for free/cheap (no spare parts needed). Restores function but adds a "Fragile" trait (higher break chance, lower efficiency). Fragile stacks.
**Emergence:** The entire power grid is a ticking time bomb of jury-rigged fuses because you never have time to fix it properly.
**Tension:** Fix it right (Time/Cost) vs. Fix it now (Risk).

## 2. Dependencies
- `006` Building Placement
- `009` Job System

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_jury_rigging_restores_function_without_parts() {
    // Arrange: A damaged building requiring spare parts to properly repair
    // Act: Apply Jury-Rig action
    // Assert: Building function is restored, no spare parts consumed
}

#[test]
fn test_jury_rigging_adds_fragile_trait() {
    // Arrange: A damaged building
    // Act: Apply Jury-Rig action
    // Assert: Building gains `Fragile` trait
}

#[test]
fn test_fragile_trait_stacks_and_increases_break_chance() {
    // Arrange: A building with multiple `Fragile` stacks
    // Act: Process breakdown chance
    // Assert: Breakdown chance is significantly higher than a normal building
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct Fragile(u32);
// fn process_jury_rig_action(...) { ... }
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
