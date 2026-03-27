# Planet Designation

## 1. Overview
**Layer:** 2
**Fantasy:** A galactic empire needs specialized organs. A stomach, a brain, a fist.
**Mechanic:** Assign a "Designation" to a colony (e.g., "Agri-World", "Fortress World"). Grants massive bonuses to specific outputs but penalties to others. Changing it causes anarchy.
**Emergence:** You designate a "Fortress World" on your border. The border moves. Now you have a useless, angry planet full of soldiers in the middle of your empire.
**Tension:** Flexible generalist worlds vs. Efficient specialist worlds.

## 2. Dependencies
- Layer 1 Colony Resources (e.g., `ColonyResources` or similar resource management)
- Layer 2 nodes representing colonies

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_planet_designation_grants_bonuses() {
    // Arrange: Setup test data with an Agri-World designation
    // Act: Process resource generation system
    // Assert: Verify food output is significantly higher than baseline, while industrial output is penalized
}

#[test]
fn test_planet_designation_change_causes_unrest() {
    // Arrange: Setup an existing Fortress World
    // Act: Change designation to Agri-World
    // Assert: Verify a massive spike in colony unrest/anarchy is applied
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct PlanetDesignation(DesignationType);
// enum DesignationType { Agri, Forge, Fortress }
// fn process_designation_bonuses(...) { ... }
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
