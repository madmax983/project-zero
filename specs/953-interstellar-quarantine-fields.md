# 953: Interstellar Quarantine Fields

## 1. Overview

Sometimes the only way to stop a plague is to lock the door and throw away the key, dooming billions. A late-game megastructure, the "Subspace Nullifier," can be built at the edge of a star system. When activated, it projects a massive field that completely prevents FTL travel into or out of the system. The system becomes an impenetrable, isolated island. A terrifying memetic or biological plague begins sweeping the galaxy. It enters one of your most populous, wealthy core systems. In a moment of cold calculus, you activate the Subspace Nullifier. The system is saved from outside invasion, but it can never trade again, and the plague is locked inside with billions of your own citizens. You watch the system slowly tear itself apart on the Layer 2 map, knowing you can never turn the field off without risking the rest of your empire.

## 2. Dependencies

- `099` Fleet Movement
- `462` Interstellar Travel
- `152` Orbital Stations

## 3. RED Phase: Tests First

```rust
#[test]
fn test_subspace_nullifier_prevents_ftl_entry() {
    // Arrange: A star system with an active Subspace Nullifier and a fleet attempting to jump in.
    let mut app = App::new();

    // Act: Process fleet movement orders.
    app.update();

    // Assert: The fleet's jump order fails, and the fleet remains in its origin system or in transit.
}

#[test]
fn test_subspace_nullifier_prevents_ftl_exit() {
    // Arrange: A star system with an active Subspace Nullifier and a fleet attempting to jump out.
    let mut app = App::new();

    // Act: Process fleet movement orders.
    app.update();

    // Assert: The fleet's jump order fails.
}

#[test]
fn test_nullifier_blocks_trade_routes() {
    // Arrange: An active trade route traversing a system that activates a Subspace Nullifier.
    let mut app = App::new();

    // Act: Process the activation of the Nullifier and update trade networks.
    app.update();

    // Assert: Trade routes through the system are severed, and local trade ceases.
}

#[test]
fn test_nullifier_permanence() {
    // Arrange: A system with an active Subspace Nullifier.
    let mut app = App::new();

    // Act: Attempt to deactivate the Nullifier via standard player commands.
    app.update();

    // Assert: The Nullifier cannot be deactivated (or attempting to do so triggers a massive, irreversible penalty/event).
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// Create a `SubspaceNullifier` component for megastructures.
// Create an `ActiveNullifierField` component to attach to `StarSystem` entities when the nullifier is activated.
// In the `evaluate_ftl_jump_system`, check if the origin or destination `StarSystem` has an `ActiveNullifierField`. If so, reject the jump order.
// In the `trade_route_evaluation_system`, invalidate any route where a node has an `ActiveNullifierField`.
// Ensure there is no built-in system to remove the `ActiveNullifierField` once applied.
```

## 5. REFACTOR Phase: Quality & Design

- Clearly signal the "Isolated" state on the galaxy map UI so players understand why fleets are stuck.
- Generate a `ChronicleEvent` when the nullifier is activated, marking the tragic sacrifice of the system.
- The permanence of the nullifier is key to the tension. If it *can* be destroyed, it should require a massive, external military operation (e.g., a multi-fleet siege).

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Activating the Subspace Nullifier completely blocks FTL travel in and out of the system.
- [ ] Existing trade routes are severed.

## 7. Technical Guidance

- Modify `src/layer3/movement/ftl_jump.rs` to include the nullifier check.
- Modify `src/layer3/economy/trade_routes.rs` to handle route invalidation due to nullifiers.
- Consider adding a specific `IsolationPanic` morale modifier for Pops inside the quarantined system.

## 8. Questions

*Builder: add questions here if spec is unclear.*
