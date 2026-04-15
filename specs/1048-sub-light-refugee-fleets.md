# 1048: Sub-light Refugee Fleets

## 1. Overview
When a Layer 3 civilization collapses, it spawns "Sub-light Refugee Fleets." Due to lacking hyper-drive technology, these fleets take significant time (decades/centuries) to reach neighboring systems. Upon arrival, they generate an event representing the influx of archaic refugees expecting a pristine world, creating tension for the colony.

## 2. Dependencies
- Layer 2 `fleet` and `navigation` systems.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_sub_light_fleet_progress_and_arrival() {
        let mut app = App::new();
        app.add_plugins(SubLightRefugeePlugin);
        app.add_event::<RefugeeArrivalEvent>();

        // Spawn a refugee fleet 10 "distance units" away with speed 5
        let fleet = app.world_mut().spawn(SubLightRefugeeFleet {
            distance_remaining: 10.0,
            speed: 5.0,
        }).id();

        // Tick 1: fleet should move closer
        app.update();

        let fleet_state = app.world().get::<SubLightRefugeeFleet>(fleet).unwrap();
        assert_eq!(fleet_state.distance_remaining, 5.0, "Fleet should move 5 units.");

        // No arrival event yet
        let events = app.world().resource::<Events<RefugeeArrivalEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_none(), "Arrival event should not fire yet.");

        // Tick 2: fleet arrives
        app.update();

        // Verify arrival event was emitted
        let events = app.world().resource::<Events<RefugeeArrivalEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some(), "Arrival event should fire when distance reaches 0.");

        // Verify fleet entity is despawned
        assert!(app.world().get::<SubLightRefugeeFleet>(fleet).is_none(), "Fleet should be despawned upon arrival.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/fleet/refugees.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct SubLightRefugeeFleet {
    pub distance_remaining: f32,
    pub speed: f32,
}

#[derive(Event, Debug)]
pub struct RefugeeArrivalEvent {
    pub fleet_entity: Entity,
}

pub struct SubLightRefugeePlugin;

impl Plugin for SubLightRefugeePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RefugeeArrivalEvent>()
           .add_systems(Update, refugee_fleet_movement_system);
    }
}

fn refugee_fleet_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SubLightRefugeeFleet)>,
    mut arrival_events: EventWriter<RefugeeArrivalEvent>,
) {
    for (entity, mut fleet) in query.iter_mut() {
        fleet.distance_remaining -= fleet.speed;
        if fleet.distance_remaining <= 0.0 {
            arrival_events.send(RefugeeArrivalEvent { fleet_entity: entity });
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Multiply `speed` by a `SimulationTime` delta if one exists, to make movement frame-independent.
- Connect the `RefugeeArrivalEvent` to Layer 1 `Pop` generation, so refugees actually spawn in the colony.
- Track fleet origin to adjust the culture or tech-level of the incoming refugees.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_sub_light_fleet_progress_and_arrival` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- Sub-light fleets should be distinct from normal `Fleet`s to avoid logic entanglements with fast-moving FTL ships.
- The arrival event should eventually integrate with the `Chronicle` system to log the historic event.
## 8. Questions
*Builder: add questions here if spec is unclear.*
