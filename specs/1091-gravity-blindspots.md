# Spec 1091: Gravity Blindspots

## 1. Overview
The "Gravity Blindspots" feature implements stealth mechanics on the Layer 2 System Map. Celestial bodies (Stars, Planets, Moons, Gas Giants) will now cast "Sensor Shadows" or act as Line-of-Sight blockers. Ships attempting to scan or detect other fleets will not be able to "see" them if a large gravity well is directly between the scanner and the target. This enables ambush tactics and requires players to make strategic choices between solar exposure and stealth.

## 2. Dependencies
- Layer 2 System Map (`SystemMap` or equivalent positioning system).
- `Ship` and `Fleet` entities with detection radius components (`SensorRange`).
- `CelestialBody` entities (Planets, Stars) with physical radii.

## 3. RED Phase: Tests First

```rust
// tests/gravity_blindspots_tests.rs
use bevy::prelude::*;
use crate::layer2::ships::{Ship, Fleet, SensorRange, VisibilityStatus};
use crate::layer2::celestial::{CelestialBody, CelestialRadius};
use crate::layer2::espionage::sensor_occlusion_system;

#[test]
fn test_celestial_body_blocks_line_of_sight() {
    let mut app = App::new();
    app.add_systems(Update, sensor_occlusion_system);

    // Arrange: Spawn scanning ship
    let scanner_ship = app.world_mut().spawn((
        Ship,
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        SensorRange { radius: 500.0 },
    )).id();

    // Arrange: Spawn target ship (behind the planet)
    let target_ship = app.world_mut().spawn((
        Ship,
        Transform::from_translation(Vec3::new(-100.0, 0.0, 0.0)),
        VisibilityStatus { is_visible: true },
    )).id();

    // Arrange: Spawn planet in the middle blocking LoS
    app.world_mut().spawn((
        CelestialBody,
        Transform::from_translation(Vec3::ZERO),
        CelestialRadius { radius: 50.0 },
    ));

    // Act: Run the occlusion system
    app.update();

    // Assert: The target ship should no longer be visible to the scanner
    let visibility = app.world().get::<VisibilityStatus>(target_ship).unwrap();
    assert!(!visibility.is_visible, "Target ship should be occluded by the celestial body.");
}

#[test]
fn test_unobstructed_line_of_sight() {
    let mut app = App::new();
    app.add_systems(Update, sensor_occlusion_system);

    // Arrange: Spawn scanning ship
    let scanner_ship = app.world_mut().spawn((
        Ship,
        Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
        SensorRange { radius: 500.0 },
    )).id();

    // Arrange: Spawn target ship
    let target_ship = app.world_mut().spawn((
        Ship,
        Transform::from_translation(Vec3::new(-100.0, -100.0, 0.0)),
        VisibilityStatus { is_visible: false },
    )).id();

    // Arrange: Spawn planet off to the side (not blocking)
    app.world_mut().spawn((
        CelestialBody,
        Transform::from_translation(Vec3::new(100.0, -100.0, 0.0)),
        CelestialRadius { radius: 50.0 },
    ));

    // Act: Run the occlusion system
    app.update();

    // Assert: The target ship should become visible
    let visibility = app.world().get::<VisibilityStatus>(target_ship).unwrap();
    assert!(visibility.is_visible, "Target ship should be visible when unobstructed.");
}
```

## 4. GREEN Phase: Minimal Implementation
- Update/Create `src/layer2/espionage.rs` or `src/layer2/sensors.rs`.
- Define `VisibilityStatus { pub is_visible: bool }` if it doesn't already exist.
- Implement `sensor_occlusion_system`:
  1. Iterate over all scanning entities (e.g., ships/stations with `SensorRange`).
  2. For each scanner, iterate over all potential target entities.
  3. Calculate the line segment from scanner to target.
  4. Iterate over all `CelestialBody` entities with `CelestialRadius`.
  5. Check for line-sphere intersection between the LoS segment and each celestial body.
  6. If an intersection occurs, the target is occluded (set `is_visible = false`). If no bodies occlude it and it is within `SensorRange`, set `is_visible = true`.

## 5. REFACTOR Phase: Quality & Design
- **Performance:** Doing line-sphere intersections for every pair of scanner/target against every celestial body is an $O(S \times T \times C)$ operation. Consider using spatial partitioning (like a grid, quadtree, or broad-phase bounding box checks) if entity counts grow large.
- **Visuals:** Provide visual feedback in the UI showing "Sensor Shadows" behind planets based on the active player fleet's position.
- **Edge Cases:** Handle cases where a ship is *partially* occluded or sitting directly inside a gravity well.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new sensor occlusion logic.
- [ ] Ships behind planets relative to scanners are hidden.

## 7. Technical Guidance
- **Math:** To check if a line segment intersects a sphere, compute the closest point on the line segment to the sphere's center, and check if the distance from the center to that point is less than the sphere's radius.
- **Stealth Tagging:** Ensure `VisibilityStatus` interacts correctly with rendering and AI targeting systems. Invisible fleets should not be drawn or targetable by automated systems.

## 8. Questions
*Builder: Add questions here if the mathematical implementation of the line-sphere intersection is unclear or if integrating with existing UI visibility toggles presents issues.*
