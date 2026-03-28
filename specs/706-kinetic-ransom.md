# 706 - The Kinetic Ransom

## 1. Overview
An enemy doesn't need to invade if they can just hold a rock over your head. A hostile Layer 2 fleet captures a massive asteroid and tows it into a precarious, decaying orbit directly above your Layer 1 colony. They demand a massive tribute. If you don't pay (or if you attack them and fail), they cut the tow lines and the asteroid falls, causing a deadly shower of meteorites that bombards your colony for a week, forcing everyone underground and destroying surface agriculture.

## 2. Dependencies
- `099-fleet-movement` (Layer 2 Fleets)
- `184-orbital-debris` (For the meteor shower impact effects on Layer 1)
- `039-trade-system` or `010-chronicle-system` (for paying tribute / history)
- `159-fleet-combat` (for the option to attack the fleet)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_kinetic_ransom_event_spawns_layer2_entity() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_kinetic_ransom);

        schedule.run(&mut world);

        // Check if the ransom entity exists
        let mut ransom_count = 0;
        for (entity, ransom) in world.query::<(Entity, &KineticRansom)>().iter(&world) {
            ransom_count += 1;
            assert!(ransom.time_remaining > 0.0, "Ransom must have a countdown");
            assert!(ransom.tribute_cost > 0.0, "Ransom must have a cost");
        }
        assert_eq!(ransom_count, 1, "A Kinetic Ransom entity should be spawned");
    }

    #[test]
    fn test_paying_ransom_despawns_entity_and_deducts_resources() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            credits: 5000.0,
            ..Default::default()
        });

        let ransom_entity = world.spawn(KineticRansom {
            time_remaining: 100.0,
            tribute_cost: 1000.0,
        }).id();

        world.init_resource::<Events<PayRansomEvent>>();
        let mut events = world.resource_mut::<Events<PayRansomEvent>>();
        events.send(PayRansomEvent { ransom_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_ransom_payments);

        schedule.run(&mut world);

        assert!(world.get_entity(ransom_entity).is_none(), "Ransom entity should be despawned after payment");
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.credits, 4000.0, "Tribute cost should be deducted");
    }

    #[test]
    fn test_ransom_timeout_triggers_meteor_shower() {
        let mut world = World::new();
        let ransom_entity = world.spawn(KineticRansom {
            time_remaining: 0.1,
            tribute_cost: 1000.0,
        }).id();

        world.init_resource::<Events<MeteorShowerEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_ransom_countdown);

        schedule.run(&mut world);

        // Entity should be despawned because time ran out
        assert!(world.get_entity(ransom_entity).is_none(), "Ransom entity should despawn on timeout");

        // Should trigger a meteor shower event for Layer 1
        let events = world.resource::<Events<MeteorShowerEvent>>();
        let mut reader = events.get_reader();
        let mut found = false;
        for _ in reader.read(events) {
            found = true;
        }
        assert!(found, "Meteor shower event should be triggered upon timeout");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer2/kinetic_ransom.rs

use bevy::prelude::*;
use crate::layer1::resources::ColonyResources;

#[derive(Component, Debug, Clone)]
pub struct KineticRansom {
    pub time_remaining: f32,
    pub tribute_cost: f32,
}

#[derive(Event, Debug)]
pub struct PayRansomEvent {
    pub ransom_entity: Entity,
}

#[derive(Event, Debug)]
pub struct MeteorShowerEvent {
    pub duration: f32,
    pub intensity: f32,
}

// System to randomly trigger this event (mocked for now)
pub fn trigger_kinetic_ransom(mut commands: Commands) {
    // In reality, this would be tied to a random event generator or hostile faction logic
    // For the test, we just spawn it.
    commands.spawn(KineticRansom {
        time_remaining: 300.0, // 5 minutes real time
        tribute_cost: 5000.0, // High cost
    });
}

pub fn process_ransom_payments(
    mut commands: Commands,
    mut events: EventReader<PayRansomEvent>,
    mut resources: ResMut<ColonyResources>,
    query: Query<&KineticRansom>,
) {
    for ev in events.read() {
        if let Ok(ransom) = query.get(ev.ransom_entity) {
            if resources.credits >= ransom.tribute_cost {
                resources.credits -= ransom.tribute_cost;
                commands.entity(ev.ransom_entity).despawn_recursive();
                // Optionally add a chronicle event here "We paid the extortionists."
            }
        }
    }
}

pub fn process_ransom_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KineticRansom)>,
    mut meteor_events: EventWriter<MeteorShowerEvent>,
) {
    for (entity, mut ransom) in query.iter_mut() {
        ransom.time_remaining -= time.delta_seconds();

        if ransom.time_remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();

            meteor_events.send(MeteorShowerEvent {
                duration: 600.0, // 10 minutes of meteor showers
                intensity: 1.0,  // Max intensity
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Fleet Combat:** The ransom entity should ideally be attached to or associated with a hostile Layer 2 `Fleet` entity. If that fleet is destroyed via combat (`159-fleet-combat`), the ransom entity should also be resolved. However, destroying the fleet should still trigger a partial meteor shower (as described in the spec: "the shattered asteroid becomes a deadly shower").
- **Meteor Shower Execution:** The `MeteorShowerEvent` needs to be hooked up to Layer 1 systems. It should instantiate a weather/hazard entity that randomly drops kinetic strikes (`184-orbital-debris` mechanics) over the colony map for its duration, forcing Pops indoors and damaging crops.
- **Chronicle & UI:** Add chronicle events for when the ransom appears, when it's paid, and when the asteroid falls. Ensure UI alerts the player to the countdown.

## 6. Acceptance Criteria
- [ ] `KineticRansom` entity tracks a countdown and a tribute cost.
- [ ] `PayRansomEvent` successfully deducts credits and removes the threat.
- [ ] Expiration of the countdown removes the threat entity and fires a `MeteorShowerEvent`.
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for `kinetic_ransom.rs` is ≥85%.

## 7. Technical Guidance
- Create `src/layer2/kinetic_ransom.rs`.
- Register the `PayRansomEvent` and `MeteorShowerEvent` in `src/setup.rs` and cleanup systems.
- Add systems to the main Layer 2 execution schedule.
- You do not need to implement the full meteor dropping logic in this spec if it belongs in Layer 1 weather/debris systems, but ensure the event is fired correctly so Integrator agents can wire it up later.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
