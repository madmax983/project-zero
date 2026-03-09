# 183: Geodetic Sentience

## Overview

Introduces "Living Stone", a rare resource found in deep mines that exhibits semi-sentient behavior. When unobserved (or just over time), `LivingStone` items slowly migrate towards each other or heat sources. If enough stones gather in one location (e.g., a Stockpile), they fuse into a hostile `StoneGolem` entity.

This creates a tension: You want to mine and store this valuable resource, but storing too much in one place creates a monster.

## Dependencies

- `018` — Mining Resources (source of stones)
- `022` — Resource Stockpiles (where they gather)
- `092` — Hostile Fauna (base for Golem entity)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/geodetic_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::geodetic::{LivingStone, StoneGolem, update_living_stone_system, form_golem_system};
    use crate::layer1::temperature::TemperatureGrid;
    use crate::simulation::SimulationTime;

    #[test]
    fn test_living_stone_migration() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_living_stone_system);
        world.insert_resource(SimulationTime { ticks: 1000 });
        world.insert_resource(TemperatureGrid::new(20, 20)); // Flat temp

        // Spawn two Living Stones 2 tiles apart
        let id1 = world.spawn((
            Item { item_type: ItemType::LivingStone, ..Default::default() },
            LivingStone { last_move_tick: 0 },
            GridPosition { x: 10, y: 10 },
        )).id();

        let id2 = world.spawn((
            Item { item_type: ItemType::LivingStone, ..Default::default() },
            LivingStone { last_move_tick: 0 },
            GridPosition { x: 12, y: 10 },
        )).id();

        // Run system
        schedule.run(&mut world);

        // They should move closer (Chebyshev distance)
        let pos1 = world.get::<GridPosition>(id1).unwrap();
        let pos2 = world.get::<GridPosition>(id2).unwrap();

        // Either 1 moved to (11, 10) or 2 moved to (11, 10) or both
        // Note: System might need multiple ticks or stochastic check, but for test assume determinstic attraction
        let dist = (pos1.x - pos2.x).abs() + (pos1.y - pos2.y).abs();
        assert!(dist < 2, "Stones should migrate towards each other. Dist: {}", dist);
    }

    #[test]
    fn test_living_stone_heat_attraction() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { ticks: 1000 });

        // Setup heat source at (20, 20)
        let mut temp_grid = TemperatureGrid::new(30, 30);
        temp_grid.set_temp(20, 20, 100.0);
        world.insert_resource(temp_grid);

        let id = world.spawn((
            Item { item_type: ItemType::LivingStone, ..Default::default() },
            LivingStone { last_move_tick: 0 },
            GridPosition { x: 15, y: 15 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_living_stone_system);

        schedule.run(&mut world);

        let pos = world.get::<GridPosition>(id).unwrap();
        // Should move towards (20, 20) i.e., x increases, y increases
        assert!(pos.x > 15 || pos.y > 15, "Stone should move towards heat");
    }

    #[test]
    fn test_golem_formation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(form_golem_system);

        // Spawn 5 Living Stones at the same location (e.g., in a stockpile)
        for _ in 0..5 {
            world.spawn((
                Item { item_type: ItemType::LivingStone, ..Default::default() },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 5, y: 5 },
            ));
        }

        // Run system
        schedule.run(&mut world);

        // Check for Golem entity
        let golem_count = world.query::<&StoneGolem>().iter(&world).len();
        assert_eq!(golem_count, 1, "Should form one Golem");

        // Check stones are consumed
        let stone_count = world.query::<&LivingStone>().iter(&world).len();
        assert_eq!(stone_count, 0, "Stones should be consumed");
    }

    #[test]
    fn test_living_stone_in_inventory_merge() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(form_golem_system);

        // Spawn a Stockpile building at (5, 5) with Inventory containing 5 Living Stones
        // Note: This assumes Inventory system stores items as entities or data we can query.
        // For this test, we assume the system can "see" stones inside inventory at the building's location.

        let stockpile = world.spawn((
            GridPosition { x: 5, y: 5 },
            // Inventory component with 5 LivingStone items
            // ... (Mock inventory setup)
        )).id();

        // If items are entities parented to stockpile:
        for _ in 0..5 {
            world.spawn((
                Item { item_type: ItemType::LivingStone, ..Default::default() },
                LivingStone { last_move_tick: 0 },
                bevy_ecs::hierarchy::Parent(stockpile), // Parented to building
                // Note: No GridPosition on item itself, system must resolve parent position
            ));
        }

        // Run system
        schedule.run(&mut world);

        // Expect Golem at (5, 5)
        let golem_count = world.query::<(&StoneGolem, &GridPosition)>().iter(&world)
            .filter(|(_, pos)| pos.x == 5 && pos.y == 5)
            .count();
        assert_eq!(golem_count, 1, "Stones in inventory should fuse into Golem at building location");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components (`src/layer1/geodetic.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::items::{Item, ItemType};
use crate::simulation::SimulationTime;
use crate::layer1::temperature::TemperatureGrid;
use rand::Rng;
use std::collections::HashMap;

#[derive(Component, Default)]
pub struct LivingStone {
    pub last_move_tick: u64,
}

#[derive(Component)]
pub struct StoneGolem {
    pub hp: i32,
    pub max_hp: i32,
}

pub const GOLEM_THRESHOLD: usize = 5;
pub const MOVE_INTERVAL: u64 = 100; // Ticks

pub fn update_living_stone_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &mut GridPosition, &mut LivingStone, &Item)>,
    other_stones: Query<&GridPosition, With<LivingStone>>,
    temp_grid: Res<TemperatureGrid>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut pos, mut living, item) in query.iter_mut() {
        if time.ticks < living.last_move_tick + MOVE_INTERVAL {
            continue;
        }

        if item.item_type != ItemType::LivingStone {
            continue;
        }

        living.last_move_tick = time.ticks;

        // Logic: Find nearest other stone or heat
        // 1. Check Heat
        let current_temp = temp_grid.get_temp(pos.x, pos.y);
        // Find neighbor with highest temp
        let mut best_move = None;
        let mut max_temp = current_temp;

        // Simple neighbor check
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                // Bounds check omitted for brevity in spec
                let t = temp_grid.get_temp(nx, ny);
                if t > max_temp {
                    max_temp = t;
                    best_move = Some((nx, ny));
                }
            }
        }

        // 2. Attraction to other stones (override heat if close?)
        // Calculate center of mass of nearby stones?
        // ... (Implementation detail)

        if let Some((nx, ny)) = best_move {
             pos.x = nx;
             pos.y = ny;
        }
    }
}

pub fn form_golem_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition), With<LivingStone>>,
) {
    let mut map: HashMap<(i32, i32), Vec<Entity>> = HashMap::new();

    for (e, pos) in query.iter() {
        map.entry((pos.x, pos.y)).or_default().push(e);
    }

    for ((x, y), stones) in map {
        if stones.len() >= GOLEM_THRESHOLD {
            // Despawn stones
            for e in stones {
                commands.entity(e).despawn();
            }
            // Spawn Golem
            commands.spawn((
                StoneGolem { hp: 100, max_hp: 100 },
                GridPosition { x, y },
                // Add Hostile, Fauna, etc.
            ));
        }
    }
}
```

### 2. Item Type Update (`src/layer1/items.rs`)

Add `LivingStone` to `ItemType` enum.

## REFACTOR Phase: Quality & Design

- **Spatial Hashing**: `form_golem_system` is O(N) with the hashmap, which is good.
- **Stockpile Interaction**: Ensure stones in `Stockpile` (which might remove `GridPosition` or hide it) are still counted. If `Item` is stored in `Inventory` of a building, query that too.
- **Movement Physics**: Ensure stones don't phase through walls. Use `Pathfinder` if necessary, or keep it "creepy" (phasing).
- **Narrative**: Add log entry when Golem forms ("The rocks are moving...").
- **Inventory Breakout**: If a stone is inside a `Stockpile` inventory, it needs to "pop out" to merge or merge inside. The system should query `Inventory` components too.

## Acceptance Criteria

- [ ] `LivingStone` items slowly move towards each other on the ground.
- [ ] `LivingStone` items move towards high temperature zones.
- [ ] Stacking 5+ `LivingStone` items spawns a `StoneGolem`.
- [ ] The Golem is hostile (basic attack logic or tag).
- [ ] `LivingStone` can be mined/spawned.

## Technical Guidance

- Use `ItemType::LivingStone` variant.
- Verify `GridPosition` is present on items. If items are in a `Stockpile` building, they might not have `GridPosition` but be inside `Inventory`. The system must handle "Items in Inventory" logic:
    - If in Inventory, they effectively share the Building's position.
    - They might "break out" of the inventory (remove from inventory, spawn on ground) to merge.

## Questions

- Do they move if carried by a Pop? (No, Pop movement overrides).
- Can they fuse inside a Pop's inventory? (Fun, but maybe too complex for MVP. Stick to Ground/Stockpile).
  - *Architect:* No, fusion should only occur on the ground or in stockpiles for the MVP.
