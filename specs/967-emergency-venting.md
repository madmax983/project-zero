# 967: Emergency Venting

## 1. Overview
The "Emergency Venting" feature serves as the ultimate fire extinguisher. It provides manual control to force open airlocks or vents, rapidly decreasing local atmospheric pressure to a vacuum state. This instantly extinguishes fires and sucks out hazardous gases/smoke, but carries the inherent risk of sucking out unanchored items or Pops and causing cold/suffocation damage.

## 2. Dependencies
- `010` Pops
- `090` Building Placement (Airlocks/Vents)
- `100` Fire (from `src/layer1/nature/fire.rs`)
- `110` Pressure/Atmosphere Grid (`PressureGrid` from `src/layer1/physics/pressure.rs`)
- `120` Control Systems (`DoorControl` from `src/layer1/core/control.rs`)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_emergency_venting_extinguishes_fire() {
    // Arrange: Set up a closed room with a Fire and an Airlock.
    let mut app = App::new();

    // Act: Force the DoorControl of the Airlock to DoorState::Open to vent the room.
    app.update();

    // Assert: Verify that the Fire entity is despawned or extinguished due to lack of pressure.
}

#[test]
fn test_emergency_venting_causes_vacuum_damage() {
    // Arrange: Place a Pop in a pressurized room with an Airlock.
    let mut app = App::new();

    // Act: Force the Airlock open (DoorState::Open). Run the update loop multiple times to ensure the room vents into a vacuum.
    app.update();

    // Assert: Verify the Pop takes damage from lack of pressure/suffocation.
}

#[test]
fn test_emergency_venting_pulls_unanchored_items() {
    // Arrange: Place an unanchored ResourceItem in a pressurized room with an Airlock.
    let mut app = App::new();

    // Act: Force the Airlock open.
    app.update();

    // Assert: The unanchored item's position changes, moving towards the venting airlock (suction effect).
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Integrate `DoorControl` with the pressure/venting systems. When an Airlock or Gate has `DoorState::Open`, the `process_door_venting_system` drops pressure heavily at its coordinates.
// The existing `fire_pressure_check_system` in `src/layer1/nature/fire.rs` should already despise vacuum and extinguish fires. Ensure it runs after the venting system.
// Implement a basic suction mechanic: when rapid pressure changes occur (delta threshold), find unanchored entities (ResourceItems, Pops) and pull them toward the vacuum source.
```

## 5. REFACTOR Phase: Quality & Design
- Centralize pressure delta tracking so suction mechanics don't require expensive full-grid scans; perhaps flag specific coordinates during massive venting.
- Ensure the `suction_system` has safety checks to avoid infinite loops or pushing entities through walls.
- Add visual indicators (UI/chronicle event) warning the player of a "Rapid Decompression" event to provide immediate feedback on the consequences.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Forcing an airlock open rapidly vents the room to vacuum.
- [ ] Fires are instantly extinguished in the vacuumed area.
- [ ] Pops in the vented area take suffocation damage.

## 7. Technical Guidance
- The `DoorControl` component in `src/layer1/core/control.rs` already supports `DoorState::Open`. The primary missing piece is the violent suction effect and ensuring pressure drops fast enough.
- Check `fire_pressure_check_system` to confirm its integration with the `PressureGrid`.
- Add integration logic in `src/layer1/physics/suction.rs` or a new file to handle pulling unanchored entities.

## 8. Questions
*Builder: Add questions here if the spec is unclear regarding suction distance or damage thresholds.*
