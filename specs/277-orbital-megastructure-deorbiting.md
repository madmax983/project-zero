# Spec 277: Orbital Megastructure Deorbiting

## 1. Overview
Massive Layer 2 structures can be caught by constructing tether arrays on Layer 1. Catching them grants tech/salvage, failing destroys the map.

## 2. Dependencies
- `152` Orbital Stations
- `017` Designation System
- `184` Orbital Debris

## 3. RED Phase: Tests First

```rust
#[test]
fn test_megastructure_deorbit_warning() {
    // Arrange: Setup orbital megastructure
    // Act: Trigger deorbit event
    // Assert: Warning is logged and time remaining is set
}

#[test]
fn test_successful_catch_grants_salvage() {
    // Arrange: Setup deorbiting megastructure, tether arrays built
    // Act: Advance time to impact
    // Assert: Impact avoided, massive tech/resources added to stockpile
}

#[test]
fn test_failed_catch_destroys_map() {
    // Arrange: Setup deorbiting megastructure, no tethers
    // Act: Advance time to impact
    // Assert: Impact occurs, destroying buildings and pops in radius
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal `DeorbitingEvent`
// `TetherArray` building required to catch
// Salvage reward or massive explosive damage on impact
```

## 5. REFACTOR Phase: Quality & Design
- Integrate the impact with the `Meteor` or `VolatileExplosion` damage types.
- Ensure the salvage is tied to the `TechState` to grant advanced technology.
- Create a UI element to show the time to impact and tether progress.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Megastructures either crash destructively or grant massive rewards if tethered.

## 7. Technical Guidance
- Create `src/layer1/events/deorbit.rs` and `src/layer1/buildings/tether_array.rs`.
- Hook into the `AddChronicleEvent` for both successful catches and catastrophic failures.
- The impact should be an event dispatched across the ECS.

## 8. Questions
*Builder: add questions here if spec is unclear.*
