# 951: Space Barnacles

## 1. Overview

The hull needs scraping. Nature clutches at your ankles. Void-fauna attach to ships over time (especially when traveling slowly or through Nebulae). They add parasitic mass, which significantly reduces the ship's base movement speed, and they drain power from the ship's reserves. Players must periodically remove them. Removal requires either dedicating time for an Extra-Vehicular Activity (EVA) operation or attempting a dangerous "Atmospheric Dipping" maneuver that burns them off but risks damaging the ship's structural integrity.

## 2. Dependencies

- `099` Fleet Movement — `specs/099-fleet-movement.md`

## 3. RED Phase: Tests First

```rust
#[test]
fn test_barnacle_accumulation_in_nebula() {
    // Arrange: App with a fleet in a Nebula hex, advancing simulation time.
    let mut app = App::new();

    // Act: Advance time by multiple ticks.
    app.update();

    // Assert: The fleet's SpaceBarnacles component has increased its mass/stack value.
}

#[test]
fn test_barnacle_mass_slows_ship() {
    // Arrange: A fleet with accumulated SpaceBarnacles.
    let mut app = App::new();

    // Act: Calculate the fleet's modified movement speed.
    app.update();

    // Assert: The actual speed is lower than the fleet's base speed due to the barnacle mass.
}

#[test]
fn test_barnacle_power_drain() {
    // Arrange: A fleet with SpaceBarnacles and a finite energy reserve.
    let mut app = App::new();

    // Act: Advance time.
    app.update();

    // Assert: The energy reserve is depleted by an amount proportional to the barnacle stack size.
}

#[test]
fn test_barnacle_removal_via_eva() {
    // Arrange: A fleet with SpaceBarnacles, commanded to perform an EVA Scrub operation.
    let mut app = App::new();

    // Act: Process the EVA operation over the required time duration.
    app.update();

    // Assert: SpaceBarnacles are removed and the fleet's speed is restored to normal, but the fleet was stationary during the process.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// Create a `SpaceBarnacles` component with `mass: f32` and `drain_rate: f32`.
// In a `barnacle_accumulation_system`, check if a Fleet is moving through a `Nebula` tile. If so, increment `SpaceBarnacles.mass`.
// Modify the `calculate_fleet_speed` system or function to subtract a speed penalty based on `SpaceBarnacles.mass`.
// In `fleet_upkeep_system`, deduct energy reserves based on `SpaceBarnacles.drain_rate`.
// Create an `EvAScrubOrder` that halts fleet movement for N ticks and resets `SpaceBarnacles` to zero.
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the speed modifier logic integrates cleanly with existing `FleetComposition` speed constraints (e.g., the slowest ship defines the fleet's speed, so barnacles on one transport affect the whole fleet).
- Prevent `SpaceBarnacles` mass from making a ship's speed negative. Enforce a minimum floor (e.g., 10% of base speed).
- Consider making "Atmospheric Dipping" a discrete command that removes barnacles instantly but applies structural damage to all ships in the fleet.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Barnacles properly reduce fleet movement speed and deplete energy.
- [ ] EVA Scrub successfully restores speed while consuming time.

## 7. Technical Guidance

- Place the systems in a new module: `src/layer2/barnacles.rs`.
- `SpaceBarnacles` should likely be attached to individual `Ship` entities within the `Fleet`, but you can accumulate them on the `Fleet` entity for the MVP.
- Ensure the power drain logic gracefully handles cases where the fleet's energy reserve reaches zero (e.g., triggering a stranded/distress state).

## 8. Questions

*Builder: add questions here if spec is unclear.*
