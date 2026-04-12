# 968: Gravity Fishing

## 1. Overview
Reeling in a big one from the junkyard of orbit. This feature introduces "Gravity Harpoons" that can lock onto passing Layer 2 debris or derelicts. Once locked, they slowly winch the object down to a designated "Drop Zone" on Layer 1. This requires massive power and comes with significant risks: the cable might snap (causing the debris to crash randomly and destructively) or you might accidentally pull down active threats like Pirate Boarding Pods.

## 2. Dependencies
- `090` Building Placement
- `115` Power Grid
- `150` Layer 2 Orbital Entities

## 3. RED Phase: Tests First
```rust
#[test]
fn test_gravity_harpoon_winching_debris() {
    // Arrange: Set up a Layer 1 Gravity Harpoon with sufficient power and a Layer 2 Debris entity.
    let mut app = App::new();

    // Act: Fire the harpoon and advance simulation time to winch the debris.
    app.update();

    // Assert: The debris descends to the Layer 1 Drop Zone successfully.
}

#[test]
fn test_gravity_harpoon_cable_snap() {
    // Arrange: Set up a harpoon winching an overly massive Layer 2 entity.
    let mut app = App::new();

    // Act: Advance simulation time.
    app.update();

    // Assert: The cable snaps, the debris crashes at a random Layer 1 coordinate, dealing AOE damage.
}

#[test]
fn test_gravity_harpoon_power_drain() {
    // Arrange: Set up a harpoon and activate it.
    let mut app = App::new();

    // Act: Process the power grid tick.
    app.update();

    // Assert: The harpoon consumes a massive amount of power from the connected grid while active.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Create a `GravityHarpoon` building component with `target_entity` and `winch_progress`.
// Create a `winch_execution_system` that checks if the harpoon is powered. If powered, it increments `winch_progress` towards 100%.
// Once 100% is reached, transition the Layer 2 entity to a Layer 1 Drop Zone via an integration event.
// Add a chance for `cable_snap` based on the mass of the target entity vs. the harpoon's rating, causing an orbital strike event on failure.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure integration between Layer 2 entity representations and Layer 1 impact logic is clean (e.g., using a bridge event `OrbitalDropEvent`).
- Polish the UI/visual feedback: add tension warnings if a cable is about to snap so the player can abort the operation.
- Balance the power consumption so it requires a dedicated grid setup for larger hauls.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Harpoons can successfully pull debris from Layer 2 to Layer 1.
- [ ] Harpoons correctly fail and cause catastrophic crashes if the target is too heavy or RNG fails.
- [ ] Harpoons consume power heavily during operation.

## 7. Technical Guidance
- Integration between layers will likely require looking at `src/layer1/core/integration.rs` or creating a specific bridge for orbital pulls.
- `PowerGrid` components in `src/layer1/infrastructure/power.rs` will dictate the energy costs.

## 8. Questions
*Builder: Add questions here if spec is unclear regarding cable snap probabilities or power formulas.*
