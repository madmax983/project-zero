# 1327: Gravimetric Graveyards

**Layer:** 2
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** The eeriness of navigating through ancient, crushing starship tombs, seeking treasure where physics itself has broken down.

**Mechanic:** Some destroyed super-capital ships or massive orbital stations don't just leave debris; they create localized gravity distortions. These "Gravimetric Graveyards" crush normal ships but contain pristine, ancient technology. Specialized "Grav-Tug" salvage ships must be built to navigate the warped space.

---

## 2. Dependencies

- `001` Main Loop & ECS Scaffold
- `011` Resources

---

## 3. RED Phase: Tests First

```rust
#[test]
fn test_gravimetric_graveyard_damages_normal_ships() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, run_graveyard_damage_system);

    let graveyard = app.world_mut().spawn((
        GravimetricGraveyard { intensity: 10.0 },
    )).id();

    let normal_ship = app.world_mut().spawn((
        scale::layer2::fleet::Fleet,
        scale::layer2::fleet::InOrbit { parent: graveyard },
        scale::layer2::fleet::FleetHealth { current: 100.0, max: 100.0 },
    )).id();

    // Act
    app.update();

    // Assert
    let health = app.world().get::<scale::layer2::fleet::FleetHealth>(normal_ship).unwrap();
    assert!(health.current < 100.0, "Normal ship should take damage in a gravimetric graveyard");
}

#[test]
fn test_gravimetric_graveyard_does_not_damage_grav_tugs() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, run_graveyard_damage_system);

    let graveyard = app.world_mut().spawn((
        GravimetricGraveyard { intensity: 10.0 },
    )).id();

    let grav_tug = app.world_mut().spawn((
        scale::layer2::fleet::Fleet,
        scale::layer2::fleet::InOrbit { parent: graveyard },
        scale::layer2::fleet::FleetHealth { current: 100.0, max: 100.0 },
        GravTug,
    )).id();

    // Act
    app.update();

    // Assert
    let health = app.world().get::<scale::layer2::fleet::FleetHealth>(grav_tug).unwrap();
    assert_eq!(health.current, 100.0, "GravTug should not take damage in a gravimetric graveyard");
}
```

---

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, InOrbit, FleetHealth};

#[derive(Component)]
pub struct GravimetricGraveyard {
    pub intensity: f32,
}

#[derive(Component)]
pub struct GravTug;

pub fn run_graveyard_damage_system(
    graveyard_query: Query<&GravimetricGraveyard>,
    mut fleet_query: Query<
        (&InOrbit, &mut FleetHealth),
        (With<Fleet>, Without<GravTug>),
    >,
) {
    for (orbit, mut health) in fleet_query.iter_mut() {
        if let Ok(graveyard) = graveyard_query.get(orbit.parent) {
            health.current -= graveyard.intensity;
        }
    }
}
```

---

## 5. REFACTOR Phase: Quality & Design

- Use `Time` resource to apply damage over time (DPS) instead of flat damage per tick.
- Hook into the existing `ShipDestroyedEvent` or debris generation systems.
- Consider what happens if a Graveyard gains more mass (as per the design idea).

---

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Normal fleets take damage in Graveyards.
- [ ] Fleets with `GravTug` are immune to Graveyard damage.

---

## 7. Technical Guidance

- Integrate with `layer2::fleet` module.
- Ensure that `GravTug` can be added to standard fleets or represents a specific `ShipType` (e.g., adding a new variant to `ShipType`). Since `ShipType` is an enum, `GravTug` might act as a special component on the fleet instead, or require a new enum variant. Choose the simplest path for the RED phase.

---

## 8. Questions

*Builder: add questions here if spec is unclear.*
