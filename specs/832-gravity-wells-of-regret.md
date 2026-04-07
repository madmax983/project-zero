# Specification 832: Gravity Wells of Regret

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Your past mistakes literally orbit your planet, waiting to fall.
**Mechanic:** Failed orbital constructions or massive battles leave behind "Heavy Debris" in unstable orbits. These slowly decay and crash into Layer 1 as "Anomalous Meteors" creating craters that sometimes yield exotic salvage or hazardous anomalies.
**Emergence:** A player might deliberately destroy enemy ships in low orbit to "mine" the resulting crash sites on the surface, essentially weaponizing their own planet's gravity to secure resources at the cost of surface devastation.
**Tension:** Keeping orbit clean (safe) vs. letting debris fall for potential salvage (dangerous).

## 2. Dependencies
- Layer 1 TerrainGrid (`src/layer1/terrain.rs`)
- Layer 2 Orbit mechanics (if any, or just `HeavyDebris` component)

## 3. RED Phase: Tests First

```rust
// tests/integration/gravity_wells.rs

use bevy::prelude::*;
use scale::layer2::debris::{HeavyDebris, OrbitalDecay, process_orbital_decay};
use scale::layer1::terrain::{TerrainGrid, TerrainType};
use scale::layer1::meteor::{AnomalousMeteorEvent, handle_meteor_impacts};

#[test]
fn test_debris_decays_over_time() {
    let mut app = App::new();
    app.add_systems(Update, process_orbital_decay);

    let debris = app.world_mut().spawn(HeavyDebris {
        mass: 100.0,
        altitude: 1000.0,
        decay_rate: 10.0,
    }).id();

    app.update();

    let d = app.world().get::<HeavyDebris>(debris).unwrap();
    assert!(d.altitude < 1000.0, "Debris altitude should decay");
}

#[test]
fn test_debris_crashes_and_creates_meteor_event() {
    let mut app = App::new();
    app.add_event::<AnomalousMeteorEvent>();
    app.add_systems(Update, process_orbital_decay);

    app.world_mut().spawn(HeavyDebris {
        mass: 100.0,
        altitude: 5.0, // Close to surface
        decay_rate: 10.0,
    });

    app.update();

    let events = app.world().resource::<Events<AnomalousMeteorEvent>>();
    let reader = events.get_cursor();
    assert!(reader.len(&events) > 0, "Meteor event should be fired when altitude <= 0");
}

#[test]
fn test_meteor_impact_alters_terrain() {
    let mut app = App::new();
    app.add_event::<AnomalousMeteorEvent>();
    app.insert_resource(TerrainGrid::new(10, 10));
    app.add_systems(Update, handle_meteor_impacts);

    app.world_mut().send_event(AnomalousMeteorEvent {
        mass: 100.0,
        impact_target: (5, 5),
    });

    app.update();

    let grid = app.world().resource::<TerrainGrid>();
    assert_eq!(grid.get_tile(5, 5), Some(TerrainType::Crater), "Impact should create a crater");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/debris.rs
use bevy::prelude::*;
use crate::layer1::meteor::AnomalousMeteorEvent;

#[derive(Component, Debug, Clone)]
pub struct HeavyDebris {
    pub mass: f32,
    pub altitude: f32,
    pub decay_rate: f32,
}

pub fn process_orbital_decay(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HeavyDebris)>,
    mut meteor_events: EventWriter<AnomalousMeteorEvent>,
) {
    for (entity, mut debris) in query.iter_mut() {
        debris.altitude -= debris.decay_rate;
        if debris.altitude <= 0.0 {
            meteor_events.send(AnomalousMeteorEvent {
                mass: debris.mass,
                impact_target: (0, 0), // hardcoded for minimal pass
            });
            commands.entity(entity).despawn();
        }
    }
}

// src/layer1/meteor.rs
use bevy::prelude::*;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

#[derive(Event, Debug, Clone)]
pub struct AnomalousMeteorEvent {
    pub mass: f32,
    pub impact_target: (i32, i32),
}

pub fn handle_meteor_impacts(
    mut events: EventReader<AnomalousMeteorEvent>,
    mut grid: ResMut<TerrainGrid>,
) {
    for event in events.read() {
        grid.set_tile(event.impact_target.0, event.impact_target.1, TerrainType::Crater);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Coordinates**: The impact coordinate in `process_orbital_decay` should be randomly determined based on planet rotation or predefined trajectory.
- **Salvage vs Hazard**: `handle_meteor_impacts` should have a chance to spawn `ExoticSalvage` or `HazardousAnomaly` items in the crater.
- **Performance**: Ensure we don't process too much debris at once.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Debris altitude decays and fires an event on reaching zero.
- [ ] The event causes a terrain change to `Crater`.

## 7. Technical Guidance
- `TerrainType::Crater` might need to be added to the enum in `src/layer1/terrain.rs`. Make sure it is handled in rendering.
- Consider adding `ExoticSalvage` to `ItemType`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
