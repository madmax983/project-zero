# 376: The Schism

## Overview

"A colony divided cannot stand. The tragic realization that your neighbors are now your enemies."

**The Schism** is a Layer 1 feature where extreme faction unhappiness leads to secession. Instead of merely protesting, a faction whose happiness drops too low will claim a cluster of buildings (Housing + Workplaces) as their territory. They will lock doors to outsiders and hoard resources. This forces the player to either negotiate or use force to retake the buildings.

This creates tension: Use force to retake the buildings (combat/damage) or negotiate (concessions)?

## Dependencies

- `068` — Pop Factions (for tracking faction happiness)
- `113` — Social Stratification (for understanding class divides and clusters)

## RED Phase: Tests First

Write these tests in `src/layer1/faction/schism_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::faction::{Faction, FactionHappiness};
    use crate::layer1::faction::schism::{SchismEvent, SecededTerritory, process_schism_system};
    use crate::layer1::building::Building;

    #[test]
    fn test_low_happiness_triggers_schism() {
        let mut world = World::new();
        world.init_resource::<Events<SchismEvent>>();

        let faction = world.spawn((
            Faction { name: "Miners' Union".to_string() },
            FactionHappiness { level: 5.0 }, // Extremely low happiness
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_schism_system);
        schedule.run(&mut world);

        let schism_events = world.resource::<Events<SchismEvent>>();
        let mut reader = schism_events.get_reader();
        assert!(reader.read(schism_events).any(|e| e.faction == faction));
    }

    #[test]
    fn test_schism_claims_territory() {
        let mut world = World::new();
        world.init_resource::<Events<SchismEvent>>();

        let faction = world.spawn((
            Faction { name: "Scientists".to_string() },
            FactionHappiness { level: 0.0 },
        )).id();

        let building = world.spawn(Building).id();

        // Simulate schism event processing which should claim buildings
        world.send_event(SchismEvent { faction });

        let mut schedule = Schedule::default();
        // Assuming a system handles the event and claims territory
        schedule.add_systems(crate::layer1::faction::schism::handle_schism_event_system);
        schedule.run(&mut world);

        // Building should now be marked as seceded
        assert!(world.get::<SecededTerritory>(building).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

Implement this minimal logic in `src/layer1/faction/schism.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::faction::{Faction, FactionHappiness};
use crate::layer1::building::Building;

#[derive(Event)]
pub struct SchismEvent {
    pub faction: Entity,
}

#[derive(Component)]
pub struct SecededTerritory {
    pub faction: Entity,
}

pub fn process_schism_system(
    query: Query<(Entity, &FactionHappiness), With<Faction>>,
    mut events: EventWriter<SchismEvent>,
) {
    for (entity, happiness) in query.iter() {
        if happiness.level <= 10.0 {
            events.send(SchismEvent { faction: entity });
        }
    }
}

pub fn handle_schism_event_system(
    mut events: EventReader<SchismEvent>,
    mut commands: Commands,
    buildings: Query<Entity, With<Building>>,
) {
    for event in events.read() {
        // MVP: Claim the first available building
        if let Some(building) = buildings.iter().next() {
            commands.entity(building).insert(SecededTerritory { faction: event.faction });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The MVP simply claims the first available building. A robust implementation needs a pathfinding or clustering algorithm to claim a contiguous block of buildings (Housing + Workplaces) associated with the faction members.
- The threshold for `SchismEvent` (currently hardcoded to `10.0`) should be configurable, perhaps via a `SchismThreshold` resource or tied to specific game difficulty settings.
- We need to implement the visual and UI cues indicating a building is now locked/seceded.
- Add components like `LockedDoor` to prevent non-faction pops from entering seceded buildings.

## Acceptance Criteria

- [ ] `SchismEvent` is fired when a faction's happiness drops below a critical threshold.
- [ ] Buildings can be tagged with `SecededTerritory` to indicate they are controlled by a rebel faction.
- [ ] `process_schism_system` correctly identifies unhappy factions.
- [ ] `handle_schism_event_system` correctly claims at least one building for the faction.
- [ ] Test coverage for the new module is >= 85%.
- [ ] `cargo test` and `cargo clippy -- -D warnings` pass.

## Technical Guidance

- Ensure `SchismEvent` is properly registered in the app builder (`app.add_event::<SchismEvent>()`).
- Seceding buildings should probably interrupt any current work or tasks being performed by non-faction members inside them.
- Consider how the player will interact with `SecededTerritory` (e.g., a "Negotiate" button or "Attack" command on the building UI).

## Questions

*Builder: add questions here if spec is unclear.*
