# 1326: Hard-Light Architecture

## 1. Overview
**Layer:** 1

**Fantasy:** Walls made of light.

**Mechanic:** Projectors create physical walls/bridges. They are instant to toggle on/off. They consume power. Zero HP (pass-through) if power fails.

**Emergence:** You use Hard-Light dams to hold back a lava flow. A brownout flickers the dam for 0.1 seconds. The lava gets through.

**Tension:** Flexibility/Speed vs. Fragility (Power dependence).

## 2. Dependencies
- ECS (`bevy_ecs`)
- Power grid system
- Building/Structure system (`src/layer1/architecture/structure.rs`)

## 3. RED Phase: Tests First

```rust
// tests/hard_light_tests.rs
use bevy::prelude::*;

#[test]
fn test_hard_light_active_when_powered() {
    let mut app = App::new();
    app.add_systems(Update, hard_light_system);

    let projector_id = app.world_mut().spawn((
        HardLightProjector { powered: true },
        Structure { current_hp: 0.0, max_hp: 100.0 }, // Base structure HP
        Collider { solid: false },
    )).id();

    app.update();

    let collider = app.world().get::<Collider>(projector_id).unwrap();
    assert!(collider.solid, "Powered hard-light should be solid.");
    let structure = app.world().get::<Structure>(projector_id).unwrap();
    assert_eq!(structure.current_hp, 100.0, "Powered hard-light should have full HP.");
}

#[test]
fn test_hard_light_fails_when_unpowered() {
    let mut app = App::new();
    app.add_systems(Update, hard_light_system);

    let projector_id = app.world_mut().spawn((
        HardLightProjector { powered: false },
        Structure { current_hp: 100.0, max_hp: 100.0 },
        Collider { solid: true },
    )).id();

    app.update();

    let collider = app.world().get::<Collider>(projector_id).unwrap();
    assert!(!collider.solid, "Unpowered hard-light should not be solid.");
    let structure = app.world().get::<Structure>(projector_id).unwrap();
    assert_eq!(structure.current_hp, 0.0, "Unpowered hard-light should drop to 0 HP.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/hard_light.rs
use bevy::prelude::*;
use crate::layer1::architecture::structure::Structure;

#[derive(Component)]
pub struct HardLightProjector {
    pub powered: bool,
}

#[derive(Component)]
pub struct Collider {
    pub solid: bool,
}

pub fn hard_light_system(
    mut query: Query<(&HardLightProjector, &mut Structure, &mut Collider)>,
) {
    for (projector, mut structure, mut collider) in query.iter_mut() {
        if projector.powered {
            collider.solid = true;
            structure.current_hp = structure.max_hp; // Restores instantly when powered
        } else {
            collider.solid = false;
            structure.current_hp = 0.0; // Fails instantly when unpowered
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding Updates:** When a collider becomes solid/non-solid, we need to trigger an update to the pathfinding grid so Pops know they can walk through unpowered walls.
- **Fluid Dynamics:** If acting as a dam, zeroing out the collider should allow the fluid map update system to push liquids (water/lava) into the tile immediately.

## 6. Acceptance Criteria
- [ ] All RED tests pass.
- [ ] Coverage >= 85%.
- [ ] HardLight walls toggle solidity based on power state.
- [ ] Unpowered hardlight immediately drops to 0 current_hp.

## 7. Technical Guidance
- `Structure` tracks integrity via `current_hp` and `max_hp`. Directly modifying these handles the integrity requirements.
- Depending on the fluid simulation, emitting a grid update event might be required when state toggles.

## 8. Questions
*Builder: Add any questions here.*
