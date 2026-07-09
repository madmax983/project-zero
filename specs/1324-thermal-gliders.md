# 1324: Thermal Gliders

## 1. Overview
**Layer:** 1

**Fantasy:** Riding the heat of industry.

**Mechanic:** "Glider Haulers" consume no fuel but require "Updrafts" from Heat sources (Furnaces, Vents) to gain altitude/speed. Cold zones ground them.

**Emergence:** Your logistics network relies on the heat from the smelters. When the smelters shut down for maintenance, the gliders are grounded, and the supply chain collapses.

**Tension:** Centralized heat (Glider highways) vs. Dispersed cooling (Safety).

## 2. Dependencies
- ECS (`bevy_ecs`)
- Thermal/Pressure Grid system (`src/layer1/map.rs`, `src/layer1/terrain.rs`)
- Movement system

## 3. RED Phase: Tests First

```rust
// tests/thermal_glider_tests.rs
use bevy::prelude::*;

#[test]
fn test_glider_moves_in_updraft() {
    let mut app = App::new();
    app.add_systems(Update, glider_movement_system);

    // Glider over hot zone
    let glider_id = app.world_mut().spawn((
        Glider { speed: 0.0, grounded: true },
        GridPosition { x: 5, y: 5 },
        ThermalTile { heat: 100.0 }, // Hot updraft
    )).id();

    app.update();

    let glider = app.world().get::<Glider>(glider_id).unwrap();
    assert!(!glider.grounded, "Glider should not be grounded over high heat.");
    assert!(glider.speed > 0.0, "Glider should gain speed in an updraft.");
}

#[test]
fn test_glider_grounded_in_cold() {
    let mut app = App::new();
    app.add_systems(Update, glider_movement_system);

    // Glider over cold zone
    let glider_id = app.world_mut().spawn((
        Glider { speed: 5.0, grounded: false },
        GridPosition { x: 5, y: 5 },
        ThermalTile { heat: -10.0 }, // Cold
    )).id();

    app.update();

    let glider = app.world().get::<Glider>(glider_id).unwrap();
    assert!(glider.grounded, "Glider should be grounded over cold zones.");
    assert_eq!(glider.speed, 0.0, "Grounded glider should have zero speed.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/thermal_glider.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct Glider {
    pub speed: f32,
    pub grounded: bool,
}

#[derive(Component)]
pub struct ThermalTile {
    pub heat: f32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub fn glider_movement_system(
    mut glider_query: Query<(&mut Glider, &ThermalTile)>,
) {
    for (mut glider, thermal) in glider_query.iter_mut() {
        if thermal.heat > 50.0 {
            glider.grounded = false;
            glider.speed += 1.0;
        } else if thermal.heat <= 0.0 {
            glider.grounded = true;
            glider.speed = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Map Integration:** `ThermalTile` shouldn't be attached to the Glider entity; the glider should sample the `AtmosphereGrid` or `PressureGrid` at its current `GridPosition`.
- **Momentum:** Gliders shouldn't instantly ground. They should lose altitude/speed gradually in cold zones until they hit zero.

## 6. Acceptance Criteria
- [ ] All RED tests pass.
- [ ] Coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Gliders gain speed over hot tiles.
- [ ] Gliders lose speed and become grounded over cold tiles.

## 7. Technical Guidance
- Integrate correctly with the temperature maps in `src/layer1/terrain.rs`.
- Gliders should probably be a special type of logistics hauler, so ensure they integrate cleanly with existing pathfinding, but respect heat paths over distance paths.

## 8. Questions
*Builder: Add any questions here.*
