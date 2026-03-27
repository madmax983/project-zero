# Indoctrination

## 1. Overview
**Layer:** 1
**Fantasy:** Shaping the minds of the next generation. 1984 meets The Sims.
**Mechanic:** Schools and Media Stations broadcast "Ethics". Pops exposed to them slowly shift their Ethics to match the State's. High alignment = Stability/Zeal. Low alignment = Dissent.
**Emergence:** You try to brainwash a captured pirate population into being "Pacifists". It backfires, and they convert your teachers to "Militarism" instead.
**Tension:** Free Thought (Innovation/Chaos) vs. State Ideology (Stability/Stagnation).

## 2. Dependencies
- Layer 1 Pop traits/ethics (`Memories`, `UtilityWeights`)
- Buildings that emit localized effects (`Schools`, `MediaStations`)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_indoctrination_shifts_pop_ethics_over_time() {
    // Arrange: Pop with divergent ethics near an active Media Station broadcasting State Ethics
    // Act: Step the simulation forward over time
    // Assert: Pop's ethics drift closer to the State Ethics
}

#[test]
fn test_indoctrination_failure_causes_dissent() {
    // Arrange: Pop with highly stubborn, opposed ethics exposed to Indoctrination
    // Act: Step simulation
    // Assert: Pop gains Dissent/Unrest instead of shifting ethics, potentially spreading it to the facility
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct IndoctrinationAura { target_ethic: Ethic, strength: f32 }
// fn process_indoctrination_system(...) { ... }
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
