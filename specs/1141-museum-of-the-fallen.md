# Spec 1141: The Museum of the Fallen

## 1. Overview
Honoring the failures and tragedies of your early colony becomes a source of later strength. A unique "Museum" building can only be constructed over the exact tiles where a major disaster occurred (e.g. mass starvation, structural collapse). It converts negative historical memories of Pops into a powerful "Resilience" mood buff.

## 2. Dependencies
- `crate::layer1::map::GridPosition`
- `crate::layer1::building::Building`
- `crate::layer1::memory::Memories`
- `crate::layer1::morale::Morale`

## 3. RED Phase: Tests First
```rust
#[test]
fn test_museum_requires_disaster_tile() {
    let mut app = App::new();
    // TODO: Setup world, map, and a tile with a disaster history

    // Attempt to build Museum on normal tile -> fails

    // Attempt to build Museum on disaster tile -> succeeds
}

#[test]
fn test_museum_converts_negative_memory_to_resilience() {
    let mut app = App::new();
    // TODO: Setup world, pop with negative memory, and a Museum

    // Verify pop gains Resilience buff when interacting with Museum
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Ensure proper integration with existing memory and mood systems.
- Consider performance implications of checking tile history during building placement.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement a `DisasterHistory` component or resource to track tiles where disasters occurred.
- Create a `Museum` building type that checks for `DisasterHistory` during placement.
- Add a system for pops to interact with the museum and gain the `Resilience` buff by updating their `Morale`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
