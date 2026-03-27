# Sensor Ambiguity

## 1. Overview
**Layer:** 2
**Fantasy:** The tension of submarine warfare. Starring at a blip on the radar, praying it's just a glitch.
**Mechanic:** Unidentified objects on the System Map appear as generic "Contacts" with a "Signal Strength". Is it a pirate? A merchant? An asteroid? You have to fly closer (risk) or hail them (reveal yourself) to find out. High-tech sensors identify contacts at longer ranges.
**Emergence:** You ignore a "weak signal" thinking it's space junk. It turns out to be a stealth frigate that nukes your orbital station.
**Tension:** Investigate (safety/risk) vs. Ignore (economy/risk).

## 2. Dependencies
- Layer 2 Map and Fleets
- Fleet vision / sensor range system

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_distant_fleet_appears_as_unidentified_contact() {
    // Arrange: Fleet A with basic sensors, Fleet B far away
    // Act: Evaluate sensor contacts for Fleet A
    // Assert: Fleet B is detected as an `UnidentifiedContact` with basic signal strength, not a full `Fleet` entity
}

#[test]
fn test_close_proximity_reveals_contact_identity() {
    // Arrange: Fleet A moves close to an `UnidentifiedContact`
    // Act: Evaluate sensor contacts
    // Assert: Contact is resolved into its true identity (e.g. `PirateFleet`)
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct SensorContact { signal_strength: f32, resolved_entity: Option<Entity> }
// fn resolve_sensors_system(...) { ... }
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
