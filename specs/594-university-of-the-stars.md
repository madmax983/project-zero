# 594: University of the Stars

## 1. Overview
**Layer:** 3 -> 1
**Fantasy:** Knowledge is power, and you are the librarian.
**Mechanic:** High-level Education buildings attract "Foreign Students" (Pops from other Civs). They pay tuition (Credits/Diplomacy) but bring their home Civ's Ethics (Influence pressure).

## 2. Dependencies
- 012-input-architecture.md
- 004-pop-entity.md

## 3. RED Phase: Tests First
```rust
#[test]
fn test_university_attracts_foreign_students() {
    // Arrange: Build high-level education
    // Act: Advance time
    // Assert: Foreign student pops arrive with tuition and foreign ethics
}

#[test]
fn test_foreign_students_exert_influence() {
    // Arrange: Foreign students present in colony
    // Act: Advance time
    // Assert: Local pops shift towards foreign ethics over time
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add ForeignStudent component
// Implement system to trigger student arrivals based on education tier
// Implement system to apply ethic drift from ForeignStudent to local pops
```

## 5. REFACTOR Phase: Quality & Design
- Consolidate ethic drift calculations into a reusable helper.
- Extract tuition payment logic to a generic resource exchange module.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Foreign students successfully arrive, pay tuition, and cause ethic drift

## 7. Technical Guidance
- Integrate with existing Pop and Ethics modules in Layer 1.
- Tie tuition payments to the economic cycle.

## 8. Questions
*Builder: add questions here if spec is unclear.*
