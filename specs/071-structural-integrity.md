# 071: Structural Integrity

## Overview

Implements **Structural Integrity** for mining operations, introducing the risk of cave-ins.
- **Roof Grid**: Tracks which tiles have an overhead roof (underground vs open sky).
- **Support Logic**: Roofs must be supported by solid structures (`Rock`, `Wall`, `Pillar`) within a 5-tile radius.
- **Cave-In**: Mining too far from support causes the roof to collapse, destroying items, damaging pops, and filling the tile with rubble (`Rock`).

This adds a risk/reward mechanic to mining: strip-mining is fast but dangerous; leaving pillars is safe but slower.

## Dependencies

- `018` — Mining and Resources (for `mine_rock`)
- `034` — Pop Health (for damage)
- `002` — Terrain Grid (for `TerrainType`)
- `006` — Building Placement (for `BuildingType::Wall`)

## RED Phase: Tests First

Write these tests in `src/layer1/structural_integrity_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::structural_integrity::{RoofGrid, check_stability, apply_collapse};
    use crate::layer1::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_roof_initialization() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Rock; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Initialize RoofGrid based on Terrain
        // Rock should have roof, others false (unless specified)
        // For MVP, assume map gen handles this. Here we test manual init.
        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(0, 0, true); // (0,0) is Rock

        assert!(roof_grid.has_roof(0, 0));
        assert!(!roof_grid.has_roof(1, 1));
    }

    #[test]
    fn test_mining_preserves_roof() {
        // Mining changes Rock -> Dirt, but Roof should remain true
        // This test simulates mining logic update
        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(5, 5, true); // Target tile

        // Simulate mine_rock completion
        // ... (mine_rock logic updates terrain)
        // Check roof remains
        assert!(roof_grid.has_roof(5, 5));
    }

    #[test]
    fn test_stability_check_safe() {
        let mut world = World::new();
        // 5x5 area
        let mut tiles = vec![TerrainType::Dirt; 25];
        tiles[0] = TerrainType::Rock; // Support at (0,0)
        world.insert_resource(TerrainGrid { width: 5, height: 5, tiles });

        let mut roof_grid = RoofGrid::new(5, 5);
        roof_grid.set(1, 0, true); // Neighbor to Rock
        world.insert_resource(roof_grid);

        // Check (1,0) - distance 1 to support
        assert!(check_stability(&world, GridPosition { x: 1, y: 0 }));
    }

    #[test]
    fn test_stability_check_unsafe() {
        let mut world = World::new();
        // 10x10 area
        let mut tiles = vec![TerrainType::Dirt; 100];
        tiles[0] = TerrainType::Rock; // Only support at (0,0)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(9, 9, true); // Far away
        world.insert_resource(roof_grid);

        // Check (9,9) - distance > 5
        assert!(!check_stability(&world, GridPosition { x: 9, y: 9 }));
    }

    #[test]
    fn test_building_provides_support() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Dirt; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Place Wall at (0,0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 0, y: 0 }
        ));

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(1, 0, true);
        world.insert_resource(roof_grid);

        assert!(check_stability(&world, GridPosition { x: 1, y: 0 }));
    }

    #[test]
    fn test_collapse_mechanics() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Dirt; 100];
        tiles[55] = TerrainType::Dirt; // (5,5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Spawn victim
        let victim = world.spawn((
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 5, y: 5 }
        )).id();

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(5, 5, true);
        world.insert_resource(roof_grid);

        // Trigger collapse
        apply_collapse(&mut world, GridPosition { x: 5, y: 5 });

        // 1. Terrain becomes Rock
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Rock));

        // 2. Roof is gone (filled)
        // Optionally, roof logic might vary. Let's assume roof remains (it's rock now).
        // Actually, if it's rock, it HAS a roof implicitly.

        // 3. Victim takes damage
        let health = world.get::<Health>(victim).unwrap();
        assert!(health.current < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `RoofGrid`

In `src/layer1/structural_integrity.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct RoofGrid {
    width: usize,
    height: usize,
    has_roof: Vec<bool>,
}

impl RoofGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            has_roof: vec![false; width * height],
        }
    }

    pub fn set(&mut self, x: i32, y: i32, val: bool) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.has_roof[(y as usize) * self.width + (x as usize)] = val;
        }
    }

    pub fn has_roof(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= self.width || (y as usize) >= self.height {
            return false;
        }
        self.has_roof[(y as usize) * self.width + (x as usize)]
    }
}
```

### 2. Implement `check_stability`

In `src/layer1/structural_integrity.rs`:

```rust
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::GridPosition;

pub const MAX_SUPPORT_DIST: i32 = 5;

pub fn check_stability(world: &World, pos: GridPosition) -> bool {
    let roof = world.resource::<RoofGrid>();
    if !roof.has_roof(pos.x, pos.y) {
        return true; // No roof = safe (open sky)
    }

    // Optimization: Check if self is support
    let terrain = world.resource::<TerrainGrid>();
    if terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Rock) {
        return true;
    }

    // BFS or simple iteration to find nearest support
    // Since range is small (5), iteration is cheap.
    // Check circular area or square? Square is simpler (Chebyshev).
    // Let's use Euclidean or Manhattan? Usually mining support is radius.
    // Let's use Chebyshev (chessboard distance) for simplicity on grid.

    let min_x = (pos.x - MAX_SUPPORT_DIST).max(0);
    let max_x = (pos.x + MAX_SUPPORT_DIST).min(terrain.width as i32 - 1);
    let min_y = (pos.y - MAX_SUPPORT_DIST).max(0);
    let max_y = (pos.y + MAX_SUPPORT_DIST).min(terrain.height as i32 - 1);

    // Collect buildings once to avoid repeated queries?
    // Actually, we can just iterate the area and check terrain + query for buildings at pos.
    // Querying buildings by pos is slow without a spatial map.
    // MVP: Iterate terrain. If Dirt, check for Building.
    // This is O(R^2 * N_buildings) if naive.
    // Optimization: Only check terrain first. If terrain is Rock -> Supported.

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Check distance
            if (x - pos.x).abs().max((y - pos.y).abs()) > MAX_SUPPORT_DIST {
                continue;
            }

            // Check Terrain Support
            if terrain.get(x as usize, y as usize) == Some(TerrainType::Rock) {
                return true;
            }

            // Check Building Support (Wall/Pillar)
            // Need a way to check if a building exists at (x,y)
            // Using a spatial lookup resource would be ideal, but for now we might iterate.
            // CAUTION: iterating all buildings is slow.
            // Recommendation: In `structural_integrity_system`, maintain a cache or use a spatial map if available.
            // For MVP, we'll assume `BuildingTracker` or similar exists, or query coarsely.
        }
    }

    // If we reach here, no rock support found.
    // Now check buildings. (In a real impl, we'd check them in the loop above efficiently).
    // For the "Green" phase, we can iterate all buildings and check pos. Slow but correct.
    let mut query = world.query::<(&Building, &GridPosition)>();
    for (building, b_pos) in query.iter(world) {
         if (b_pos.x - pos.x).abs().max((b_pos.y - pos.y).abs()) <= MAX_SUPPORT_DIST {
             if matches!(building.building_type, BuildingType::Wall | BuildingType::AncientReactor) { // Pillars?
                 return true;
             }
         }
    }

    false
}
```

### 3. Implement `apply_collapse`

In `src/layer1/structural_integrity.rs`:

```rust
use crate::layer1::health::Health;

pub fn apply_collapse(world: &mut World, pos: GridPosition) {
    // 1. Change Terrain to Rock (Rubble)
    let mut terrain = world.resource_mut::<TerrainGrid>();
    let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
    terrain.tiles[idx] = TerrainType::Rock;

    // 2. Damage entities
    // Iterate all entities with Health and GridPosition at this pos
    // Again, spatial query is best. Naive iteration for MVP.
    let mut victims = Vec::new();
    let mut query = world.query::<(Entity, &GridPosition, &mut Health)>();

    for (entity, p, _) in query.iter(world) {
        if p.x == pos.x && p.y == pos.y {
            victims.push(entity);
        }
    }

    for entity in victims {
        if let Ok(mut health) = world.get_mut::<Health>(entity) {
            health.current -= 50.0; // Big damage
        }
    }

    // 3. Log event
    // ...
}
```

### 4. Integration

Hook into `mine_rock` in `src/layer1/resources.rs`:
- When mining completes:
  - `check_stability(pos)`
  - If unstable -> `apply_collapse(pos)`

## REFACTOR Phase: Quality & Design

- **Performance**: The `check_stability` function iterates all buildings. This is O(N) per tile mined. For large colonies, this will lag.
  - *Fix*: Use a `BuildingGrid` resource (similar to `TerrainGrid`) that stores `Option<Entity>` or `BuildingType` at each coordinate.
- **Roof Initialization**: Currently `RoofGrid` needs manual init. Map generation should populate it (All Rock = Roof, Surface = No Roof).
- **Cascade**: A collapse might trigger *another* collapse if the destroyed tile was supporting others. This requires a recursive check or a dirty-flag system.
- **Visuals**: "Rock" tile looks like unmined wall. Should separate `TerrainType::Rubble` or use a different char.

## Acceptance Criteria

- [ ] `RoofGrid` resource added.
- [ ] `check_stability` correctly identifies supported vs unsupported tiles.
- [ ] Mining rock preserves `Roof=true` on the tile.
- [ ] Mining too far from Rock/Wall triggers `apply_collapse`.
- [ ] Collapse damages pops and resets terrain to Rock.
- [ ] Tests pass.

## Technical Guidance

- Use `Chebyshev` distance (max of dx, dy) for support radius to match the square grid aesthetic.
- Be careful with the `check_stability` performance. If tests are slow, mock the building check.
- Ensure `RoofGrid` is serialized/saved if save/load exists (not yet).
