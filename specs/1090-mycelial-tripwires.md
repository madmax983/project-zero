# 1090: Mycelial Tripwires

## 1. Overview
Fungal networks connect distant parts of the map. Stepping on a "Sensor Shroom" in one area alerts "Spore Turrets" or Fauna in another area to attack. The forest has eyes.

## 2. Dependencies
- Layer 1 Map / `TerrainGrid` pathing events.
- Entity movement / collision tracking.
- Fungal/Fauna entity framework (`FaunaAI`, `CombatTarget`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_stepping_on_sensor_triggers_network_alert() {
        let mut app = App::new();
        app.add_event::<MycelialTripwireEvent>();
        app.add_systems(Update, detect_tripwire_step);

        let pop = app.world_mut().spawn(GridPosition { x: 5, y: 5 }).id();
        let _sensor = app.world_mut().spawn((
            SensorShroom { network_id: 1 },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();

        let events = app.world().resource::<Events<MycelialTripwireEvent>>();
        let mut reader = events.get_reader();
        let triggered = reader.read(events).next().unwrap();

        assert_eq!(triggered.network_id, 1, "Network 1 should be alerted.");
        assert_eq!(triggered.triggering_entity, pop, "Pop entity should be identified as trigger.");
    }

    #[test]
    fn test_spore_turret_aggro_on_alert() {
        let mut app = App::new();
        app.add_event::<MycelialTripwireEvent>();
        app.add_systems(Update, aggro_network_entities);

        let pop = app.world_mut().spawn(GridPosition { x: 10, y: 10 }).id();

        let turret = app.world_mut().spawn((
            SporeTurret { network_id: 1, is_active: false, target: None },
        )).id();

        app.world_mut().send_event(MycelialTripwireEvent {
            network_id: 1,
            triggering_entity: pop,
        });

        app.update();

        let turret_comp = app.world().get::<SporeTurret>(turret).unwrap();
        assert!(turret_comp.is_active, "Turret should become active on network alert.");
        assert_eq!(turret_comp.target, Some(pop), "Turret should target the triggering entity.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct SensorShroom {
    pub network_id: u32,
}

#[derive(Component)]
pub struct SporeTurret {
    pub network_id: u32,
    pub is_active: bool,
    pub target: Option<Entity>,
}

#[derive(Event)]
pub struct MycelialTripwireEvent {
    pub network_id: u32,
    pub triggering_entity: Entity,
}

pub fn detect_tripwire_step(
    pops: Query<(Entity, &GridPosition), Without<SensorShroom>>,
    sensors: Query<(&SensorShroom, &GridPosition)>,
    mut events: EventWriter<MycelialTripwireEvent>,
) {
    for (pop_entity, pop_pos) in pops.iter() {
        for (sensor, sensor_pos) in sensors.iter() {
            if pop_pos == sensor_pos {
                events.send(MycelialTripwireEvent {
                    network_id: sensor.network_id,
                    triggering_entity: pop_entity,
                });
            }
        }
    }
}

pub fn aggro_network_entities(
    mut events: EventReader<MycelialTripwireEvent>,
    mut turrets: Query<&mut SporeTurret>,
) {
    for event in events.read() {
        for mut turret in turrets.iter_mut() {
            if turret.network_id == event.network_id {
                turret.is_active = true;
                turret.target = Some(event.triggering_entity);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Collision Optimization**: Checking all Pops vs all Sensors is `O(N*M)`. In practice, we should use a `SpatialHash` or trigger collision via `MovementEvent` rather than every frame.
- **Stealth**: Allow certain pops (e.g., those with a `Stealth` trait) to bypass sensors.
- **Clearing**: Allow Pops to destroy `SensorShroom` entities (which might trigger the alarm if not done correctly, introducing risk/reward).

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Entities correctly aggro onto a target who steps on a connected sensor.

## 7. Technical Guidance
- Integrate with existing AI logic: The `target` logic in `SporeTurret` should interface with the existing Combat and Utility AI systems, issuing an attack action.
- The `network_id` is crucial. The map generation system will need to generate clusters of sensors and turrets sharing the same ID.

## 8. Questions
*Builder: Add questions here if the specification is unclear.*
