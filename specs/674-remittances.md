# Remittances

## 1. Overview
**Layer:** Cross-layer
**Fantasy:** The loneliness of the migrant worker. You are here to build a better life for someone else, far away.
**Mechanic:** Pops with the "Family" trait (or from specific backgrounds) deduct a % of their earnings/resources to "send home". If they can't pay, they get "Homesick" (Depression). If they pay a lot, their home faction sends "Cousins" (new migrants).
**Emergence:** Your economy drains because everyone is sending money off-world. You ban remittances to save gold, causing a massive "Homesick" depression wave and a diplomatic incident with the homeworld.
**Tension:** Local wealth retention vs. Pop happiness/immigration.

## 2. Dependencies
- Layer 1 Pop wealth/needs (e.g. Economy domain)
- Layer 1 Morale / Needs (Depression/Homesick)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_pop_sends_remittance_and_maintains_morale() {
    // Arrange: Migrant pop with sufficient personal wealth/resources
    // Act: Process remittance cycle
    // Assert: Pop loses a percentage of wealth, 'Homesick' need remains satisfied
}

#[test]
fn test_failed_remittance_causes_homesick_depression() {
    // Arrange: Migrant pop with 0 wealth
    // Act: Process remittance cycle
    // Assert: Pop fails to send remittance, gains 'Homesick' depression mood modifier
}

#[test]
fn test_high_remittances_trigger_migrant_arrival() {
    // Arrange: High total volume of remittances sent to a specific faction
    // Act: Process diplomatic/migration triggers
    // Assert: A new `MigrantArrivalEvent` is fired from the destination faction
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct MigrantFamilyInfo { home_faction: Entity, remittance_target: f32 }
// fn process_remittances(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- List refactoring opportunities
- Identify code smells to clean up
- Document performance considerations
- Note API improvements

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Code structure suggestions
- Integration points
- Gotchas and common mistakes

## 8. Questions
*Builder: add questions here if spec is unclear.*
