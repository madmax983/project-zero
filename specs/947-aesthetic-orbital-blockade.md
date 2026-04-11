# 947: Aesthetic Orbital Blockade

## 1. Overview

The infuriating reality of ultra-wealthy orbital elites suffocating the working class for the sake of a pretty view. High-wealth Layer 2 orbital habitats rely on the aesthetic "Beauty" of the planet below for their massive mood bonuses. If Layer 1 industrial pollution or utilitarian sprawl degrades this view, the orbital elites use their political power to pass restrictive "Aesthetic Edicts," forcibly halting or sabotaging the planet's most productive factories.

## 2. Dependencies

- `152` Orbital Stations — `specs/152-orbital-stations.md`
- `054` Colony Edicts — `specs/054-colony-edicts.md`
- `113` Social Stratification — `specs/113-social-stratification.md`

## 3. RED Phase: Tests First

```rust
#[test]
fn test_high_pollution_triggers_aesthetic_edict() {
    // Arrange: App with an orbital station and high-wealth pops, and a surface with high pollution
    let mut app = App::new();
    // Setup Layer 1 pollution levels above threshold, Layer 2 OrbitalStation

    // Act: Update app to process orbital observations and politics
    app.update();

    // Assert: An 'AestheticEdict' event is emitted, or the active edicts resource is updated
    // with 'HaltHeavyIndustry'.
}

#[test]
fn test_aesthetic_edict_halts_factory_production() {
    // Arrange: Active Aesthetic Edict, and a surface factory
    let mut app = App::new();

    // Act: Process production systems
    app.update();

    // Assert: Factory production progress should not advance, and it should have a 'Halted(Aesthetic)' status.
}

#[test]
fn test_tearing_down_habitats_restores_industry() {
    // Arrange: App with halted factories and an orbital station
    let mut app = App::new();

    // Act: Destroy the orbital station (e.g., via rebel action or player command)
    // and update the app
    app.update();

    // Assert: The 'AestheticEdict' is lifted, and factory production resumes.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// Calculate total planet "Beauty" / "Pollution".
// If OrbitalStation exists and Pollution > X, add ActiveEdict::Aesthetic.
// In the work_execution_system, check if ActiveEdict::Aesthetic is present and the building is HeavyIndustry.
// If so, skip the work progress step.
```

## 5. REFACTOR Phase: Quality & Design

- Create a bridge system that evaluates Layer 1 environmental stats and pushes a summary to Layer 2 orbital stations.
- Ensure the 'Halted' status is clearly communicated to the UI so the player understands *why* their factory stopped.
- Avoid hardcoding the threshold; use a tunable constant or a value derived from the orbital pop's wealth level.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Orbital habitats halt Layer 1 industry when pollution crosses a threshold.

## 7. Technical Guidance

- Integrate with the existing `ColonyEdicts` resource.
- Consider adding a `Polluting` tag to buildings to easily filter which ones get halted.
- You may need to add an `AddChronicleEvent` when the edict is passed to notify the player.

## 8. Questions

*Builder: add questions here if spec is unclear.*
