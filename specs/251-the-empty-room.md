# 251: The "Empty" Room

## Overview

"In a crowded station, space is the ultimate luxury."

Designating a "Sanctuary" zone requires it to be **empty** (no furniture, machines, or storage). Pops visit these empty rooms to reduce Stress. The effectiveness scales with the size of the empty space.
However, entropy fights back. Pops may leave "offerings" (flowers, rocks) or clutter in the Sanctuary, breaking the "Empty" condition and disabling the bonus until cleaned.

- **Zone Requirement**: `ZoneType::Sanctuary` requires 0 buildings/items on its tiles.
- **Mechanic**: `Stress` reduction proportional to `TileCount`.
- **Emergence**: The "Janitor of the Void" loop—keeping the room empty against the wishes of the people.

## Dependencies

- `056` — Designated Zones
- `031` — Pop Morale (Stress)
- `239` — Operational Detritus (Clutter spawning)

## RED Phase: Tests First

Write these tests in `src/layer1/social/empty_room_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::zone::{Zone, ZoneType};
    use crate::layer1::map::{GridPosition, TileObject}; // Assuming object tracker
    use crate::layer1::pop::{Pop, Mood};
    use crate::layer1::social::empty_room::{Sanctuary, update_sanctuary_system, visit_sanctuary_system};

    #[test]
    fn test_sanctuary_validity_check() {
        let mut world = World::new();
        // Create Sanctuary Zone covering (0,0) and (0,1)
        let zone_ent = world.spawn((
            Zone {
                zone_type: ZoneType::Sanctuary,
                tiles: vec![GridPosition { x: 0, y: 0 }, GridPosition { x: 0, y: 1 }],
            },
            Sanctuary { is_valid: false, effectiveness: 0.0 },
        )).id();

        // Run system - should be valid as no objects exist
        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let sanctuary = world.get::<Sanctuary>(zone_ent).unwrap();
        assert!(sanctuary.is_valid);
        assert_eq!(sanctuary.effectiveness, 2.0); // 1.0 per tile
    }

    #[test]
    fn test_clutter_invalidates_sanctuary() {
        let mut world = World::new();
        let zone_ent = world.spawn((
            Zone {
                zone_type: ZoneType::Sanctuary,
                tiles: vec![GridPosition { x: 0, y: 0 }],
            },
            Sanctuary { is_valid: true, effectiveness: 1.0 },
        )).id();

        // Spawn an item/building in the zone
        world.spawn((
            TileObject, // Marker for "Thing that takes up space"
            GridPosition { x: 0, y: 0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let sanctuary = world.get::<Sanctuary>(zone_ent).unwrap();
        assert!(!sanctuary.is_valid);
        assert_eq!(sanctuary.effectiveness, 0.0);
    }

    #[test]
    fn test_visit_reduces_stress() {
        let mut world = World::new();
        let zone_ent = world.spawn((
            Zone { zone_type: ZoneType::Sanctuary, tiles: vec![] },
            Sanctuary { is_valid: true, effectiveness: 5.0 },
            GridPosition { x: 5, y: 5 }, // Center
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Inside zone
            Mood { stress: 50.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(visit_sanctuary_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop).unwrap();
        assert!(mood.stress < 50.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/social/empty_room.rs

use bevy_ecs::prelude::*;
use crate::layer1::zone::{Zone, ZoneType};
use crate::layer1::map::{GridPosition, TileObject};
use crate::layer1::pop::Mood;

#[derive(Component, Default)]
pub struct Sanctuary {
    pub is_valid: bool,
    pub effectiveness: f32,
}

pub fn update_sanctuary_system(
    mut zones: Query<(&Zone, &mut Sanctuary)>,
    objects: Query<&GridPosition, With<TileObject>>, // Query all objects
) {
    // Naive O(N*M) - optimize later with spatial map
    for (zone, mut sanctuary) in zones.iter_mut() {
        if zone.zone_type != ZoneType::Sanctuary {
            continue;
        }

        let mut occupied_count = 0;
        for tile in &zone.tiles {
            for obj_pos in objects.iter() {
                if obj_pos == tile {
                    occupied_count += 1;
                }
            }
        }

        if occupied_count == 0 {
            sanctuary.is_valid = true;
            sanctuary.effectiveness = zone.tiles.len() as f32;
        } else {
            sanctuary.is_valid = false;
            sanctuary.effectiveness = 0.0;
        }
    }
}

pub fn visit_sanctuary_system(
    mut pops: Query<(&GridPosition, &mut Mood)>,
    zones: Query<(&Zone, &Sanctuary)>,
) {
    for (pop_pos, mut mood) in pops.iter_mut() {
        for (zone, sanctuary) in zones.iter() {
            if sanctuary.is_valid && zone.contains(pop_pos) {
                mood.stress = (mood.stress - (sanctuary.effectiveness * 0.1)).max(0.0);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Spatial Optimization**: The `objects` query in `update_sanctuary_system` is slow. Use a `TileMap` resource that stores `Option<Entity>` to check occupancy in O(1).
- **Offerings**: Add a random event where `visit_sanctuary_system` has a 1% chance to spawn a `Clutter` item (Flower/Rock), invalidating the room. This creates the maintenance loop.
- **UI**: Show "Sanctuary: Invalid (Cluttered)" in Zone selection.

## Acceptance Criteria

- [ ] `Sanctuary` component tracks validity.
- [ ] Objects/Clutter inside zone invalidate it.
- [ ] Valid sanctuaries reduce stress for visitors.
- [ ] Tests pass.

## Technical Guidance

- Reuse `Zone::contains` from `056`.
- Ensure `TileObject` or equivalent covers Buildings, Items, and Debris.

## Questions

*Builder: Does a person standing in the room count as "not empty"?*
*Architect:* A Pop temporarily occupying the tile does not break emptiness, only placed structures or dropped items.
*Architect:* Pops do not invalidate the 'Sanctuary' state; only placed buildings or dropped item clutter break the emptiness.
