# Specification 870: Cultural Desync

## 1. Overview
**Layer:** Cross-layer (Layer 3 & Layer 1)
**Feature:** Cultural Desync
**Fantasy:** Distant colonies drift culturally so far that they no longer understand the core worlds.
**Mechanic:** Colonies far from the capital accumulate "Cultural Drift" over time. High drift generates new local ideologies and causes "Desync" events that disrupt interstellar trade and communication.

## 2. Dependencies
- Needs `bevy_ecs` setup.
- Layer 1 `Colony` and Layer 3 `ColonyId`/`Empire` associations.
- `Chronicle` system for logging events.

## 3. RED Phase: Tests First

```rust
// tests/cultural_desync_tests.rs
use bevy::prelude::*;
use scale::layer1::colony::Colony;
use scale::layer3::cultural_desync::{CulturalDrift, DesyncEvent, cultural_drift_system};

#[test]
fn test_cultural_drift_accumulates_based_on_distance() {
    let mut app = App::new();
    app.add_systems(Update, cultural_drift_system);

    let colony_entity = app.world_mut().spawn((
        Colony { name: "Frontier Alpha".into() },
        CulturalDrift { drift_amount: 0.0, distance_to_capital: 15.0 },
    )).id();

    app.update();

    let drift = app.world().get::<CulturalDrift>(colony_entity).unwrap();
    assert!(drift.drift_amount > 0.0);
}

#[test]
fn test_high_drift_triggers_desync_event() {
    let mut app = App::new();
    app.add_event::<DesyncEvent>();
    app.add_systems(Update, cultural_drift_system);

    app.world_mut().spawn((
        Colony { name: "Distant Outpost".into() },
        CulturalDrift { drift_amount: 99.0, distance_to_capital: 50.0 },
    ));

    app.update();

    let desync_events = app.world().resource::<Events<DesyncEvent>>();
    let mut reader = desync_events.get_cursor();
    assert!(reader.read(desync_events).count() > 0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/cultural_desync.rs
use bevy::prelude::*;
use crate::layer1::colony::Colony;

#[derive(Component)]
pub struct CulturalDrift {
    pub drift_amount: f32,
    pub distance_to_capital: f32,
}

#[derive(Event)]
pub struct DesyncEvent {
    pub colony_entity: Entity,
}

pub fn cultural_drift_system(
    mut query: Query<(Entity, &mut CulturalDrift)>,
    mut event_writer: EventWriter<DesyncEvent>,
) {
    for (entity, mut drift) in query.iter_mut() {
        // Accumulate drift based on distance
        drift.drift_amount += drift.distance_to_capital * 0.01;

        if drift.drift_amount >= 100.0 {
            event_writer.send(DesyncEvent { colony_entity: entity });
            drift.drift_amount = 0.0; // Reset after trigger
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The `DesyncEvent` should hook into the Chronicle system to generate procedural lore (`CULTURAL_DESYNC_TRIGGERED`).
- **Economy Impact**: High drift should apply a negative modifier to `TradeRoute` efficiency.
- **Counterplay**: Add a `CulturalExchange` action that players can fund to reduce `drift_amount`.

## 6. Acceptance Criteria
- [ ] Tests pass (`cargo test`).
- [ ] Coverage for new module >= 85%.
- [ ] `cultural_drift_system` correctly increases drift based on distance and fires the event at the threshold.

## 7. Technical Guidance
- Ensure distance metrics are correctly synchronized if the colony or capital moves.
- Connect the `DesyncEvent` to the UI to notify the player of ideological schisms.

## 8. Questions
*Builder: add questions here if spec is unclear.*
