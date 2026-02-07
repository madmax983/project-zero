# 043: Defensive Structures

## Overview

Introduces defensive buildings and movement blocking logic.
- **Walls**: Solid obstacles that block pathfinding and movement.
- **Gates**: Obstacles that can be toggled (Locked/Unlocked).
- **Pathfinding Update**: `is_walkable` must now check for `Building` obstacles in addition to `TerrainType`.
- **Building Health**: Buildings now have `Health` and can be destroyed.

## Dependencies

- `006` — Building Placement (for `Building` entities)
- `034` — Health System (for `Health` component)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/defense_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::execution::{movement_system, MovementTarget, AtTarget};
    use crate::layer1::utility_ai::ActionType;

    // Helper to setup world with flat grass
    fn setup_world() -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(OccupiedTiles::default());
        world
    }

    // 1. Building Obstacle Logic
    #[test]
    fn test_wall_is_obstacle() {
        let wall = Building { building_type: BuildingType::Wall };
        assert!(wall.is_obstacle());
    }

    #[test]
    fn test_gate_is_obstacle_only_when_locked() {
        // Gate requires extra component or state in Building?
        // For MVP, Gate is a BuildingType. We might need a `Gate` component.
        let mut world = World::new();
        let gate_entity = world.spawn((
            Building { building_type: BuildingType::Gate },
            GridPosition { x: 0, y: 0 },
            crate::layer1::defense::Gate { is_locked: false },
        )).id();

        // Check obstacle logic via system or helper
        assert!(!crate::layer1::defense::is_obstacle(&world, gate_entity));

        // Lock it
        world.get_mut::<crate::layer1::defense::Gate>(gate_entity).unwrap().is_locked = true;
        assert!(crate::layer1::defense::is_obstacle(&world, gate_entity));
    }

    #[test]
    fn test_normal_buildings_are_obstacles() {
        // Most buildings should be obstacles (Housing, Farm?)
        // Spec decision: Farms are walkable? Housing is obstacle?
        // Let's say: Housing = Obstacle, Farm = Walkable (crops).
        let housing = Building { building_type: BuildingType::Housing };
        assert!(housing.is_obstacle());

        let farm = Building { building_type: BuildingType::Farm };
        assert!(!farm.is_obstacle()); // Farms are walkable
    }

    // 2. Pathfinding / Movement Logic
    #[test]
    fn test_movement_blocked_by_wall() {
        let mut world = setup_world();

        // Place Wall at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
            Health::default(),
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 0));

        // Pop at (0,0) trying to move to (2,0)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MovementTarget {
                target_entity: Entity::from_raw(999),
                target_position: GridPosition { x: 2, y: 0 },
                for_action: ActionType::Idle, // Just moving
            },
        )).id();

        // Run movement
        // We need to register the system or call it.
        // But movement_system in execution.rs needs update to check obstacles.
        // For this test, we assume movement_system uses the new `is_walkable` logic.

        // We might need to inject the logic or update the system in place.
        // Since we are mocking, we can just call the updated function `is_walkable`.

        // Let's test `is_walkable` directly first.
        assert!(!crate::layer1::defense::is_walkable(&world, 1, 0));
        assert!(crate::layer1::defense::is_walkable(&world, 0, 0));
    }

    #[test]
    fn test_movement_allowed_through_open_gate() {
        let mut world = setup_world();

        // Gate at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Gate },
            GridPosition { x: 1, y: 0 },
            crate::layer1::defense::Gate { is_locked: false },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 0));

        assert!(crate::layer1::defense::is_walkable(&world, 1, 0));
    }

    // 3. Building Health & Death
    #[test]
    fn test_building_takes_damage() {
        let mut health = Health { current: 100.0, max: 100.0 };
        health.take_damage(10.0);
        assert_eq!(health.current, 90.0);
    }

    #[test]
    fn test_building_death_message() {
        // Refactor death_system to distinguish pops vs buildings
        let mut world = World::new();
        world.insert_resource(crate::shared::log::MessageLog::default());

        let wall = world.spawn((
            Building { building_type: BuildingType::Wall },
            Health { current: -1.0, max: 100.0 },
        )).id();

        crate::layer1::health::death_system(&mut world);

        assert!(world.get_entity(wall).is_err());

        let log = world.resource::<crate::shared::log::MessageLog>();
        // Should NOT say "Colonist has died"
        // Should say "Building destroyed" or nothing?
        // For now, check it doesn't say Colonist.
        // Implementation detail: we need to update death_system.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType` (`src/layer1/building.rs`)

Add `Wall`, `Gate`, `Tower`.
Add `is_obstacle()` method.

```rust
pub enum BuildingType {
    // ...
    Wall,
    Gate,
    Tower,
}

impl BuildingType {
    pub fn is_obstacle(&self) -> bool {
        match self {
            Self::Farm | Self::Stockpile => false, // Walkable
            _ => true, // Walls, Housing, etc block
        }
    }
}
```

### 2. Create `src/layer1/defense.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;

#[derive(Component, Default, Debug)]
pub struct Gate {
    pub is_locked: bool,
}

/// Checks if a tile is walkable (Terrain + Buildings).
pub fn is_walkable(world: &World, x: i32, y: i32) -> bool {
    // 1. Check Terrain
    let terrain = world.resource::<TerrainGrid>();
    if let (Ok(x_idx), Ok(y_idx)) = (usize::try_from(x), usize::try_from(y)) {
         if !terrain.get(x_idx, y_idx).is_some_and(crate::layer1::terrain::TerrainType::is_walkable) {
             return false;
         }
    } else {
        return false; // Out of bounds
    }

    // 2. Check Buildings via OccupiedTiles
    if let Some(occupied) = world.get_resource::<OccupiedTiles>() {
        if occupied.0.contains(&(x, y)) {
            // Find the building entity at this position
            // Optimization: Maintain a spatial map? For now, query.
            // This is O(N) where N is buildings. Slow for pathfinding loop!
            // TODO: Refactor OccupiedTiles to HashMap<(i32,i32), Entity> in a future PR.
            // For MVP: We assume OccupiedTiles checks are rare or map is small.
            // Actually, we can use `Query` iteration which is faster than random access if we don't have a map.
            // But `is_walkable` is called per tile.

            // Temporary Scan:
            let mut blocked = false;
            let mut buildings = world.query::<(&GridPosition, &Building, Option<&Gate>)>();
            for (pos, building, gate) in buildings.iter(world) {
                if pos.x == x && pos.y == y {
                    if let Some(g) = gate {
                        if g.is_locked { blocked = true; }
                    } else if building.building_type.is_obstacle() {
                        blocked = true;
                    }
                    break;
                }
            }
            if blocked { return false; }
        }
    }

    true
}
```

### 3. Refactor `OccupiedTiles` (Optional but recommended)

If `OccupiedTiles` was a `HashMap<(i32,i32), Entity>`, we could look up the entity instantly.
This is a good Refactor step. For Green phase, the slow scan is acceptable if tests pass.

### 4. Update `movement_system` (`src/layer1/execution.rs`)

Replace the local `is_walkable_terrain` with `defense::is_walkable`.
Note: `defense::is_walkable` requires `&World`, but `movement_system` runs in a query context.
You might need to pass `&World` or extract the logic to a helper that takes `Query`.

**Solution for System Param:**
`movement_system` cannot easily take `&World` if it has other queries.
Better: `movement_system` should query `OccupiedTiles` and `Buildings`.

```rust
// In movement_system signature:
buildings: Query<(&GridPosition, &Building, Option<&Gate>)>,

// Inside loop:
let is_walkable = |x, y| {
    // Check terrain...
    // Check buildings...
    // Use the query `buildings` to check if (x,y) has obstacle.
};
```

### 5. Update `death_system` (`src/layer1/health.rs`)

```rust
use crate::layer1::pop::Pop;

pub fn death_system(world: &mut World) {
    // ...
    for entity in to_despawn {
        let is_pop = world.get::<Pop>(entity).is_some();
        world.despawn(entity);
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            if is_pop {
                log.add("DEATH: A colonist has died!");
            } else {
                log.add("Building destroyed!");
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Change `OccupiedTiles` to `HashMap<(i32,i32), Entity>` to avoid O(N) scan in pathfinding.
- **Pathfinding**: If we add A* later, this `is_walkable` check becomes critical.
- **Visuals**: Gates need open/closed sprites.
- **Construction**: Walls should auto-connect (bitmasking) for visuals.

## Acceptance Criteria

- [ ] `Wall`, `Gate`, `Tower` added to `BuildingType`.
- [ ] `Gate` component allows locking.
- [ ] `BuildingType::is_obstacle()` implemented.
- [ ] Movement system respects building obstacles.
- [ ] Buildings have Health and can be destroyed.
- [ ] Death log distinguishes Pops from Buildings.
- [ ] Tests pass.

## Questions

- Should walls block Line of Sight (FOV)? (Yes, eventually. For now just movement).
- Do we need a "Repair" job? (Future spec).
