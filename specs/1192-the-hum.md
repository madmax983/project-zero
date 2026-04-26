# 1192: The Hum

## 1. Overview
Introduce "The Hum," a global Resonance variable on the colony station. Some Pops possess a "Sensitive" trait, enabling them to perceive this vibration. The Hum generates "Urges" prompting Sensitive Pops to travel to high-resonance zones or perform peculiar tasks. Failing to heed these urges results in increased Stress. The goal is to create emergent behavior, such as a "Cult of the Hum" forming in generator rooms, and tension around treating or listening to these Pops.

## 2. Dependencies
- Base ECS with `Pop` components and needs (Layer 1).
- Existing trait/personality system (for `Sensitive` trait).
- Needs/Stress system (`src/layer1/needs.rs`).
- Utility AI and movement systems.

## 3. RED Phase: Tests First
```rust
// specs/1192-the-hum.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_sensitive_pop_receives_urge() {
    // Arrange: Create a world with a global Resonance value.
    // Add a Pop with the 'Sensitive' trait.
    // Act: Run the system that generates Urges based on Resonance.
    // Assert: Verify the Pop has received an 'Urge' component or memory.
}

#[test]
fn test_ignoring_urge_increases_stress() {
    // Arrange: Pop has an unfulfilled Urge.
    // Act: Run the metabolism/needs decay system.
    // Assert: Verify Stress increases faster or a specific penalty is applied.
}

#[test]
fn test_fulfilling_urge_reduces_stress() {
    // Arrange: Pop with an Urge arrives at a high-resonance zone.
    // Act: Evaluate utility and execute the "Listen to Hum" action.
    // Assert: Verify the Urge is satisfied and Stress is reduced.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal code to turn tests green.
// Define a GlobalResource for Resonance.
// Define a 'Sensitive' Component/Trait.
// Define an 'Urge' Component that tracks unfulfilled hum-related desires.
// Update the Utility AI scoring to factor in 'Urge'.
// Update needs decay to penalize unfulfilled Urges.
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Avoid tight coupling between the base utility system and this specific trait. Use a generic "Urge/Compulsion" system if possible.
- **Performance**: High-resonance zones should be cached or efficiently queried, avoiding distance checks for every Sensitive Pop every tick.
- **Design**: Consider how "treating" the trait works (e.g., medication removing the trait temporarily or suppressing the stress penalty).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A Sensitive Pop will prioritize moving to a high-resonance area when an Urge triggers.
- [ ] Stress demonstrably increases if the Urge is blocked (e.g., area is unreachable).

## 7. Technical Guidance
- Integrate into the `evaluate_actions_system` to score the "Listen to Hum" action highly when the Urge is active.
- The global Resonance could fluctuate based on power usage or random events.
- To create the "Cult" emergence, you may later add a social component where Sensitive Pops discussing the Hum spread a meme/rumor.

## 8. Questions
*Builder: add questions here if spec is unclear.*
