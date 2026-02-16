# 138: The Blob

## Overview

A slow-moving, semi-indestructible hazard that consumes everything in its path.
The **Blob** is a colony-threatening entity that spreads across the map, devouring items, damaging buildings, and injuring Pops.
It serves as a late-game environmental challenge or a consequence of "digging too deep".

**Mechanics:**
- **Spread**: Blobs multiply to adjacent tiles over time.
- **Consumption**: Blobs destroy items and damage buildings on their tile.
- **Vulnerability**: Fire destroys Blob entities instantly.
- **Utility**: Can be used to dispose of waste (if contained).

## Dependencies

- `002` — Terrain Grid (GridPosition)
- `033` — Fire Propagation (Fire vulnerability)
- `045` — Structure Durability (Building damage)
- `004` — Pop Entity (Damage to pops)

## RED Phase: Tests First

Write these tests in `src/layer1/blob_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::blob::{Blob, blob_spread_system, blob_consumption_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::{Structure, HitPoints};
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::building::{Building, BuildingType};
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    #[test]
    fn test_blob_spreads_to_empty_tile() {
        let mut world = World::new();
        // Setup: 1 Blob at (5,5)
        world.spawn((
            Blob { spread_timer: 0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Mock resources (Map bounds etc) if needed
        world.insert_resource(crate::layer1::map::MapSize { width: 10, height: 10 });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Run spread system
        // We force timer to trigger
        world.run_system_once(blob_spread_system);

        // Should have more than 1 blob now
        let blob_count = world.query::<&Blob>().iter(&world).count();
        assert!(blob_count > 1, "Blob should spread");

        // Check adjacency (simplistic check)
        let positions: Vec<GridPosition> = world.query::<&GridPosition>().with::<Blob>().iter(&world).cloned().collect();
        assert!(positions.contains(&GridPosition { x: 5, y: 5 }));
        // New blob should be adjacent
        let new_pos = positions.iter().find(|p| **p != GridPosition { x: 5, y: 5 }).unwrap();
        assert!((new_pos.x - 5).abs() <= 1 && (new_pos.y - 5).abs() <= 1);
    }

    #[test]
    fn test_blob_eats_items() {
        let mut world = World::new();
        // Blob and Item at same location
        world.spawn((
            Blob::default(),
            GridPosition { x: 5, y: 5 },
        ));
        let item = world.spawn((
            Item { item_type: ItemType::Wood, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(blob_consumption_system);

        // Item should be despawned
        assert!(world.get_entity(item).is_err());
    }

    #[test]
    fn test_blob_damages_buildings() {
        let mut world = World::new();
        // Blob and Wall at same location
        world.spawn((
            Blob::default(),
            GridPosition { x: 5, y: 5 },
        ));
        let wall = world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 5, y: 5 },
            Structure { hp: HitPoints { current: 100.0, max: 100.0 }, ..Default::default() },
        )).id();

        world.run_system_once(blob_consumption_system);

        let structure = world.get::<Structure>(wall).unwrap();
        assert!(structure.hp.current < 100.0, "Blob should damage building");
    }

    #[test]
    fn test_blob_blocked_by_walls() {
        // Blobs shouldn't spread INTO a tile with a building unless it's destroyed?
        // OR: Blobs spread into walls and *then* damage them?
        // Design choice: Blobs attack walls from adjacent tiles.
        // For MVP: Blobs move onto tile -> Damage building on tile.
        // So spread checks if tile is passable?
        // Let's say Blob CANNOT coexist with Building (Stacking limit).
        // It must destroy building from outside to enter.

        // REVISIT: Spec says "Consume everything in its path".
        // Let's say: It spreads to adjacent. If Building exists, it deals damage INSTEAD of spawning.

        let mut world = World::new();
        world.insert_resource(crate::layer1::map::MapSize { width: 10, height: 10 });

        // Blob at (0,0)
        world.spawn((
            Blob { spread_timer: 0 },
            GridPosition { x: 0, y: 0 },
        ));

        // Wall at (0,1)
        let wall = world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 0, y: 1 },
            Structure { hp: HitPoints { current: 10.0, max: 10.0 }, ..Default::default() },
        )).id();

        // Mock OccupiedTiles logic
        // We need a way to tell the system (0,1) is occupied.
        // Assuming system queries buildings.

        world.run_system_once(blob_spread_system);

        // Blob count should NOT increase (blocked by wall)
        // But Wall should take damage (attacked)

        let structure = world.get::<Structure>(wall).unwrap();
        assert!(structure.hp.current < 10.0);

        let blob_count = world.query::<&Blob>().iter(&world).count();
        assert_eq!(blob_count, 1);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Blob` Component (`src/layer1/blob.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component, Default, Debug, Clone)]
pub struct Blob {
    pub spread_timer: u32, // Ticks until next spread attempt
}
```

### 2. Implement `blob_spread_system`

```rust
use rand::Rng;
use crate::layer1::building::Building;
use crate::layer1::structure::Structure;

pub fn blob_spread_system(
    mut commands: Commands,
    mut blobs: Query<(&mut Blob, &GridPosition)>,
    buildings: Query<(Entity, &GridPosition, &mut Structure), With<Building>>,
    // occupy map helper
) {
    let mut rng = rand::thread_rng();

    // 1. Build map of existing blobs to prevent stacking
    let mut blob_positions = std::collections::HashSet::new();
    for (_, pos) in blobs.iter() {
        blob_positions.insert(*pos);
    }

    // 2. Build map of buildings
    // ...

    for (mut blob, pos) in blobs.iter_mut() {
        if blob.spread_timer > 0 {
            blob.spread_timer -= 1;
            continue;
        }
        blob.spread_timer = 100; // Reset

        // Pick random neighbor
        let (dx, dy) = match rng.gen_range(0..4) {
            0 => (0, 1),
            1 => (0, -1),
            2 => (1, 0),
            _ => (-1, 0),
        };
        let target_pos = GridPosition { x: pos.x + dx, y: pos.y + dy };

        // Check bounds
        // ...

        if blob_positions.contains(&target_pos) {
            continue; // Already blobed
        }

        // Check for building
        let mut hit_building = false;
        // Optimization: Use a spatial map resource instead of iterating query
        for (b_entity, b_pos, mut structure) in buildings.iter_mut() {
            if *b_pos == target_pos {
                // Attack building!
                structure.hp.current -= 10.0;
                hit_building = true;
                break; // Only hit one
            }
        }

        if !hit_building {
            // Spawn new blob
            commands.spawn((
                Blob { spread_timer: 100 + rng.gen_range(0..50) },
                target_pos,
            ));
        }
    }
}
```

### 3. Implement `blob_consumption_system`

```rust
pub fn blob_consumption_system(
    mut commands: Commands,
    blobs: Query<&GridPosition, With<Blob>>,
    items: Query<(Entity, &GridPosition), With<crate::layer1::items::Item>>,
    mut buildings: Query<&mut Structure, With<Building>>, // On same tile? (Rare if blocked)
) {
    // If a Blob shares a tile with an Item, eat it.
    // O(N*M) naive implementation - optimize with grid map.

    for blob_pos in blobs.iter() {
        for (item_entity, item_pos) in items.iter() {
            if blob_pos == item_pos {
                commands.entity(item_entity).despawn();
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Use `GridHashMap` or similar spatial index to avoid O(N^2) checks in consumption/spread.
- **Visuals**: Blobs need a distinct color (Purple/Green) and char (`%` or `O`).
- **Fire Interaction**: Add `blob_fire_vulnerability_system` where Blobs sharing a tile with Fire die instantly.
- **Balance**: Adjust spread rate and damage.
- **Notification**: "The Blob is spreading!" warning.

## Acceptance Criteria

- [ ] `Blob` component exists.
- [ ] Blobs spread to adjacent empty tiles.
- [ ] Blobs attack buildings on adjacent tiles instead of entering.
- [ ] Blobs consume items on their tile (if they somehow overlap).
- [ ] Tests pass.

## Technical Guidance

- Create `src/layer1/blob.rs`.
- Register systems in `layer1/mod.rs`.
- Ensure spread doesn't go out of map bounds.
