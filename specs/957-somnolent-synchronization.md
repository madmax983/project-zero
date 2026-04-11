# 957: Somnolent Synchronization

## 1. Overview
If an entire sector of Pops goes to sleep simultaneously in an interconnected, high-density residential block, they enter "Somnolent Synchronization." Their individual needs and moods are averaged out across the entire group when they wake up, completely erasing individual mental breaks. However, this creates the risk of mass synchronized nightmares.

## 2. Dependencies
- `010` Pop Needs & Morale
- `022` Residential Buildings

## 3. RED Phase: Tests First
```rust
#[test]
fn test_somnolent_synchronization_triggers() {
    // Arrange: A group of Pops in a high-density residential block sleeping at the same time.
    let mut app = App::new();

    // Act: Advance time until they complete their sleep cycle.
    app.update();

    // Assert: The Pops receive the `SomnolentSynchronization` modifier, and their morale/needs are averaged.
}

#[test]
fn test_mass_nightmare_infection() {
    // Arrange: A synchronized sleep group near a Psychic Anomaly.
    let mut app = App::new();

    // Act: Process sleep events.
    app.update();

    // Assert: A `MassNightmareEvent` triggers, and all Pops in the group wake up simultaneously with massive stress/panic.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In `sleep_evaluation_system`, check if the number of sleeping pops in the same `Sector` exceeds a threshold.
// If so, tag them with `SomnolentSynchronization`.
// When they wake up (`wake_up_system`), average their `Stress` and `Morale` components and apply it evenly.
// Introduce a `nightmare_chance_system` that checks for nearby anomalies. If true, cancel sleep early, apply `Panic` to all synchronized pops, and emit `MassNightmareEvent`.
```

## 5. REFACTOR Phase: Quality & Design
- Avoid heavy O(N^2) checks for sleeping neighbors. Use a `SectorSleepTracker` resource or component on the sector entity itself.
- Ensure the averaging logic preserves integer/float types without compounding precision loss.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Waking up from synchronization perfectly averages the selected needs.
- [ ] Mass nightmares affect the entire synchronized group simultaneously.

## 7. Technical Guidance
- Place logic in `src/layer1/social/sleep_sync.rs`.
- Integrate with `src/layer1/utility_ai.rs` to handle sudden widespread panic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
