# 372: Stellar Megastructure Decay

## Overview

"Living in the ruins of gods."

**Stellar Megastructure Decay** introduces an environmental hazard for colonies built inside or near ancient megastructures (Layer 2 features impacting Layer 1). While these structures provide immense resources, they are structurally failing. "Decay Events" happen randomly—massive sections collapse, venting atmosphere or dropping debris onto your Layer 1 colonies beneath it.

This creates tension: utilizing god-like technology for incredible resource output vs. the unpredictable, apocalyptic maintenance failures that can crush half your population.

## Dependencies

- `079` — Weather Events (for managing environmental hazards)
- `184` — Orbital Debris (for the falling debris mechanics)

## RED Phase: Tests First

Write these tests in `src/layer1/hazards/megastructure_decay_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{TileType, GridPosition};
    use crate::layer1::hazards::megastructure_decay::{DecayEvent, MegastructureRuins, trigger_decay_event_system, process_debris_fall_system};
    use crate::layer1::building::{Building, Health};

    #[test]
    fn test_decay_event_triggers_randomly() {
        let mut world = World::new();
        world.init_resource::<Events<DecayEvent>>();

        // Add Megastructure marker to the map
        world.insert_resource(MegastructureRuins { stability: 0.1 });

        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_decay_event_system);
        schedule.run(&mut world);

        // Given low stability, a decay event should eventually trigger
        let events = world.resource::<Events<DecayEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }

    #[test]
    fn test_debris_damages_buildings() {
        let mut world = World::new();

        // Target building
        let pos = GridPosition { x: 5, y: 5 };
        let building = world.spawn((
            Building { building_type: crate::layer1::building::BuildingType::Housing },
            pos,
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Spawn falling debris event targeting the building
        let mut events = Events::<DecayEvent>::default();
        events.send(DecayEvent { target: pos, damage: 50.0 });
        world.insert_resource(events);

        let mut schedule = Schedule::default();
        schedule.add_systems(process_debris_fall_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(building).unwrap();
        assert_eq!(health.current, 50.0);
    }

    #[test]
    fn test_debris_alters_terrain() {
        let mut world = World::new();

        let pos = GridPosition { x: 2, y: 2 };
        let tile = world.spawn((
            pos,
            TileType::Floor,
        )).id();

        let mut events = Events::<DecayEvent>::default();
        events.send(DecayEvent { target: pos, damage: 100.0 });
        world.insert_resource(events);

        let mut schedule = Schedule::default();
        schedule.add_systems(process_debris_fall_system);
        schedule.run(&mut world);

        // Massive damage should turn floor into rubble
        let new_tile = world.get::<TileType>(tile).unwrap();
        assert_eq!(*new_tile, TileType::Rubble);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components & Resources

```rust
// src/layer1/hazards/megastructure_decay.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::{GridPosition, TileType};

#[derive(Resource, Debug, Clone)]
pub struct MegastructureRuins {
    pub stability: f32, // 0.0 to 1.0
}

#[derive(Event, Debug, Clone)]
pub struct DecayEvent {
    pub target: GridPosition,
    pub damage: f32,
}
```

### 2. Systems

```rust
use crate::layer1::building::Health;
use rand::Rng;

pub fn trigger_decay_event_system(
    ruins: Option<Res<MegastructureRuins>>,
    mut events: EventWriter<DecayEvent>,
) {
    if let Some(ruins) = ruins {
        let mut rng = rand::thread_rng();
        // Probability increases as stability decreases
        let chance = (1.0 - ruins.stability) * 0.1;
        if rng.gen::<f32>() < chance {
            events.send(DecayEvent {
                target: GridPosition { x: rng.gen_range(0..100), y: rng.gen_range(0..100) },
                damage: 50.0,
            });
        }
    }
}

pub fn process_debris_fall_system(
    mut events: EventReader<DecayEvent>,
    mut buildings: Query<(&GridPosition, &mut Health)>,
    mut tiles: Query<(&GridPosition, &mut TileType)>,
) {
    for event in events.read() {
        // Damage buildings
        for (pos, mut health) in buildings.iter_mut() {
            if *pos == event.target {
                health.current = (health.current - event.damage).max(0.0);
            }
        }

        // Alter terrain
        if event.damage >= 100.0 {
            for (pos, mut tile) in tiles.iter_mut() {
                if *pos == event.target {
                    *tile = TileType::Rubble;
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Event Warning**: Implement a delay and visual warning for the decay event before it hits, giving Pops a chance to evacuate.
- **Resource Output**: Tie the `stability` of the `MegastructureRuins` to a resource generator, so players are tempted to lower stability to extract more resources.
- **Atmospheric Venting**: Decay events should also affect the `AtmosphereGrid`, suddenly venting oxygen or introducing toxic gases.

## Acceptance Criteria

- [ ] `MegastructureRuins` resource dictates event frequency.
- [ ] `trigger_decay_event_system` fires events based on stability.
- [ ] Falling debris damages entities and changes `TileType` to `Rubble`.
- [ ] Tests pass.

## Technical Guidance

- Ensure `MegastructureRuins` is only inserted when the colony is initialized on a specific map type.
- Hook up `process_debris_fall_system` to `Layer1SystemSet::Execution`.
- Add `TileType::Rubble` if it doesn't exist, or use a suitable equivalent like `Rock`.

## Questions

*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
