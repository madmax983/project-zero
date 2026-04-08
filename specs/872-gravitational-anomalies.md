# Specification 872: Gravitational Anomalies

## 1. Overview
**Layer:** 1
**Feature:** Gravitational Anomalies
**Fantasy:** The laws of physics are broken in this valley.
**Mechanic:** Localized zones of high/low gravity on the colony map. High G causes slow movement and crushing damage over time. Low G increases movement speed and projectile range.
**Emergence:** Players might build "Jump Towers" in Low G zones to launch gliders across the map, but risk heavy haulers getting stuck in High G ruts.
**Tension:** Build in the safe Neutral zone, or the risky but useful Anomaly?

## 2. Dependencies
- Layer 1 Map & Pathfinding (`MovementSpeed`, `Position`).
- Layer 1 Health System (`Health`, `DamageEvent`).
- Layer 1 Combat/Physics (`Projectile`).
- Needs `bevy_ecs` setup.

## 3. RED Phase: Tests First

```rust
// tests/gravitational_anomalies_tests.rs
use bevy::prelude::*;
// Dummy imports for spec clarity - Builders should use actual paths
use scale::layer1::gravity::{GravityZone, apply_gravity_anomaly_system};
use scale::layer1::movement::MovementSpeed;
use scale::layer1::combat::Projectile;
use scale::layer1::health::{Health, DamageEvent};

#[test]
fn test_high_gravity_slows_movement() {
    let mut app = App::new();
    app.add_systems(Update, apply_gravity_anomaly_system);

    let pop = app.world_mut().spawn((
        MovementSpeed { value: 10.0, base: 10.0 },
        GravityZone { gravity_multiplier: 2.5 }, // High G
    )).id();

    app.update();

    let speed = app.world().get::<MovementSpeed>(pop).unwrap();
    // High gravity should reduce speed below base
    assert!(speed.value < speed.base);
}

#[test]
fn test_extreme_high_gravity_causes_damage() {
    let mut app = App::new();
    app.add_event::<DamageEvent>();
    app.add_systems(Update, apply_gravity_anomaly_system);

    let pop = app.world_mut().spawn((
        MovementSpeed { value: 10.0, base: 10.0 },
        Health { current: 100.0, max: 100.0 },
        GravityZone { gravity_multiplier: 3.5 }, // Extreme High G
    )).id();

    app.update();

    let damage_events = app.world().resource::<Events<DamageEvent>>();
    let mut reader = damage_events.get_cursor();

    // Assert that a damage event was fired due to crushing gravity
    assert!(reader.read(damage_events).next().is_some());
}

#[test]
fn test_low_gravity_increases_movement_and_projectile_range() {
    let mut app = App::new();
    app.add_systems(Update, apply_gravity_anomaly_system);

    let pop = app.world_mut().spawn((
        MovementSpeed { value: 10.0, base: 10.0 },
        GravityZone { gravity_multiplier: 0.5 }, // Low G
    )).id();

    let projectile = app.world_mut().spawn((
        Projectile { range: 50.0, base_range: 50.0 },
        GravityZone { gravity_multiplier: 0.5 }, // Low G
    )).id();

    app.update();

    let speed = app.world().get::<MovementSpeed>(pop).unwrap();
    assert!(speed.value > speed.base);

    let proj = app.world().get::<Projectile>(projectile).unwrap();
    assert!(proj.range > proj.base_range);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/gravity.rs
use bevy::prelude::*;
use crate::layer1::movement::MovementSpeed;
use crate::layer1::combat::Projectile;
use crate::layer1::health::{Health, DamageEvent};

#[derive(Component, Default, Clone, Copy)]
pub struct GravityZone {
    pub gravity_multiplier: f32, // 1.0 is standard
}

pub fn apply_gravity_anomaly_system(
    mut movement_query: Query<(&GravityZone, &mut MovementSpeed)>,
    mut projectile_query: Query<(&GravityZone, &mut Projectile)>,
    health_query: Query<(Entity, &GravityZone, &Health)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    // Apply movement modifiers
    for (gravity, mut speed) in movement_query.iter_mut() {
        speed.value = speed.base / gravity.gravity_multiplier;
    }

    // Apply projectile range modifiers
    for (gravity, mut proj) in projectile_query.iter_mut() {
        proj.range = proj.base_range / gravity.gravity_multiplier;
    }

    // Apply extreme high gravity crushing damage
    for (entity, gravity, health) in health_query.iter() {
        if gravity.gravity_multiplier >= 3.0 && health.current > 0.0 {
            damage_events.send(DamageEvent {
                target: entity,
                amount: (gravity.gravity_multiplier - 2.0) * 5.0, // Scale damage by severity
                source: None,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding**: Update the pathfinding cost heuristic so utility AI tends to avoid high-G zones unless heavily incentivized.
- **Chronicle Integration**: Add `evt_crushed_by_gravity` chronicle template when a Pop dies from Gravity crushing damage.
- **Modifiers**: Ensure gravity stacks cleanly with other speed modifiers (like encumbrance or injuries).

## 6. Acceptance Criteria
- [ ] Tests pass in RED phase.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes cleanly.
- [ ] Test coverage >= 85% for `apply_gravity_anomaly_system`.
- [ ] High gravity properly slows entities and deals damage if extreme.
- [ ] Low gravity correctly increases movement and projectile ranges.

## 7. Technical Guidance
- Gravity values can be attached to the terrain grid itself or implemented as a spatial zone component covering multiple tiles. For now, assigning `GravityZone` to entities resting on those tiles is the expected approach.
- Be careful with division by zero or near-zero gravity. Cap the minimum `gravity_multiplier` to something safe like `0.1`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
