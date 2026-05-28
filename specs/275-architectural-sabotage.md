# Spec 275: Architectural Sabotage

## 1. Overview
Rival faction contracted workers insert "Sabotage Points" into buildings during construction, which can fail later.

## 2. Dependencies
- `020` Construction Costs
- `068` Pop Factions
- `045` Structure Durability

## 3. RED Phase: Tests First

```rust
#[test]
fn test_sabotage_point_insertion() {
    // Arrange: Setup construction with rival faction builder
    // Act: Complete building
    // Assert: Building has `Sabotaged` component
}

#[test]
fn test_sabotaged_building_failure() {
    // Arrange: Setup sabotaged building
    // Act: Trigger sabotage event
    // Assert: Building explodes or collapses, damaging area
}

#[test]
fn test_vetting_prevents_sabotage() {
    // Arrange: Setup vetting policy, rival builder
    // Act: Complete building
    // Assert: Building does not have `Sabotaged` component
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal `Sabotaged` component
// System to insert `Sabotaged` based on builder faction relations
// Event to trigger failure based on time or external trigger
```

## 5. REFACTOR Phase: Quality & Design
- Create a unified `BuildingFailure` event that handles different types of explosions/collapses.
- Tie the `Sabotaged` insertion rate inversely to faction relations.
- Add a UI indicator for internal vetting modes.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Sabotaged buildings fail catastrophically when triggered.

## 7. Technical Guidance
- Create `src/layer1/building/sabotage.rs`.
- Hook into the construction completion event to insert the `Sabotaged` component.
- The failure event should trigger `VolatileExplosion` or similar damage systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
- **Architectural Contradictions:** `Sabotaged` component does not exist. `VolatileExplosion` might not exist. This spec doesn't provide enough component details to implement reliably.
