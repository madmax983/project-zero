# Spec 276: The Flesh Famine

## 1. Overview
When plant agriculture is destroyed, pops turn to meat or cannibalism, abandoning their civic ideology.

## 2. Dependencies
- `008` Farm Building
- `075` Animal Husbandry
- `197` Civic Ideology

## 3. RED Phase: Tests First

```rust
#[test]
fn test_plant_blight_destroys_crops() {
    // Arrange: Setup farms with crops
    // Act: Trigger blight
    // Assert: Crops are dead, soil fertility ruined
}

#[test]
fn test_flesh_famine_shifts_diet() {
    // Arrange: Setup starving colony with no crops, some fauna
    // Act: Advance time
    // Assert: Pops start hunting and eating fauna
}

#[test]
fn test_cannibalism_generates_taboo_and_trauma() {
    // Arrange: Setup starving colony with no crops, no fauna
    // Act: Advance time
    // Assert: Pops eat each other, gain Taboo and Trauma memories
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal `FleshFamine` event
// Diet fallback to meat and then cannibalism
// `Taboo` and `Trauma` memory insertion for cannibalism
```

## 5. REFACTOR Phase: Quality & Design
- Create a unified `DietPriority` system that gracefully degrades.
- Ensure the `Taboo` and `Trauma` memories correctly lower morale.
- Connect this to the `CivicIdeology` system to shift the ideology towards survivalist.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Starving pops eat meat or each other and gain trauma.

## 7. Technical Guidance
- Create `src/layer1/needs/diet.rs` and `src/layer1/social/flesh_famine.rs`.
- Hook into the `metabolism_system` to prioritize available food.
- Trigger `AddChronicleEvent` when cannibalism first occurs.

## 8. Questions
*Builder: add questions here if spec is unclear.*
