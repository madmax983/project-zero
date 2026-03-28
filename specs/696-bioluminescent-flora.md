# 696: Bioluminescent Flora

## 1. Overview
**Layer:** 1
**Fantasy:** The alien night is not dark; it is alive with strange lights.
**Mechanic:** Specific plants glow at night, providing small radii of light. Light reduces stress and monster spawn rates. Harvesting the plant removes the light.
**Emergence:** You clear-cut the glowing forest to build a solar farm. When night falls, the base is pitch black, and the "Shadow Stalkers" spawn in the middle of your power grid.
**Tension:** Resource extraction (wood/space) vs. Natural safety (light).

## 2. Dependencies
- `002` Terrain Grid
- `034` Pop Health (Stress)
- `161` Ecological Succession

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_bioluminescent_flora_emits_light_at_night() {
    // Arrange: Spawn a Bioluminescent Flora entity, set time to night
    // Act: Process light emission system
    // Assert: Flora entity has an active light emission component with expected radius
}

#[test]
fn test_light_reduces_stress_and_prevents_spawns() {
    // Arrange: Spawn a pop near flora and verify stress reduction, check spawn weights in light radius
    // Act: Advance simulation
    // Assert: Stress is lower than baseline, hostile spawn weights in radius are 0
}

#[test]
fn test_harvesting_flora_removes_light() {
    // Arrange: Flora entity with active light emission
    // Act: Harvest flora entity
    // Assert: Flora entity and its light emission are removed
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// #[derive(Component)]
// struct Bioluminescent;
// fn emit_light_system(...) { ... }
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
