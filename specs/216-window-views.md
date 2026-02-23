# 216: Window Views

## Overview

Introduces the **Window** component and a raycasting system to calculate view quality. Windows capture "Beauty" from distant tiles and inject it into the local `BeautyGrid` or directly to `RoomQuality`. This allows players to build for aesthetics by facing rooms towards scenic features (mountains, gardens, statues) and away from ugly ones (landfills, industry).

Currently, beauty is strictly proximity-based. You have to stand next to a statue to appreciate it. Windows allow "Beauty at a Distance", making room orientation and layout meaningful.

## Dependencies

- `044` — Horticulture & Beauty (`BeautyGrid`)
- `004` — Building System (`BuildingType`, `Direction`)
- `064` — Room Quality (Integration point)

## RED Phase: Tests First

These tests should be written in `src/layer1/window_view_tests.rs`.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid, TerrainType};
    use crate::layer1::beauty::{BeautyGrid, BeautySource};
    use crate::layer1::building::{Building, BuildingType, Direction};
    use crate::layer1::window::{Window, update_window_views_system};

    // Helper to setup world
    fn setup_world() -> World {
        let mut world = World::new();
        let width = 20;
        let height = 20;
        world.insert_resource(BeautyGrid::new(width, height));
        world.insert_resource(TerrainGrid::new(width, height));
        // Mock OccupiedTiles if needed for obstruction checks
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        world
    }

    #[test]
    fn test_window_captures_distant_beauty() {
        let mut world = setup_world();

        // 1. Place a Statue at (10, 5) with high beauty
        world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 10, y: 5 },
            BeautySource { value: 10.0, radius: 2.0 },
        ));

        // 2. Place a Window at (5, 5) facing East (towards Statue)
        world.spawn((
            Building { building_type: BuildingType::Window },
            GridPosition { x: 5, y: 5 },
            Window {
                direction: Direction::East,
                range: 10,
                view_cone: 0.0, // Straight line for simplicity
            },
            // Window itself has base beauty 0.0, but will gain "View Beauty"
            BeautySource { value: 0.0, radius: 2.0 },
        ));

        // 3. Run Update System
        // We need to run beauty update first to populate the grid from the statue?
        // Or does Window read from BeautySource components directly?
        // Window should read form the BeautyGrid (which aggregates sources).
        // So: Update Beauty Grid -> Update Windows -> Update Beauty Grid (Second Pass)?
        // For simplicity: Window reads CURRENT grid state and ADDS to it.
        // If system order is Beauty -> Window, then Window sees beauty from previous frame or current frame's static objects.

        // Let's assume we pre-populate grid with Statue beauty
        let mut grid = world.resource_mut::<BeautyGrid>();
        grid.set(10, 5, 10.0); // Manually set for test isolation

        update_window_views_system(&mut world);

        // 4. Check Window Tile Beauty
        let grid = world.resource::<BeautyGrid>();
        // Window at (5,5) should have > 0.0 beauty from the view
        assert!(grid.get(5, 5) > 0.0);
    }

    #[test]
    fn test_window_obstruction() {
        let mut world = setup_world();

        // Statue at (10, 5)
        let mut grid = world.resource_mut::<BeautyGrid>();
        grid.set(10, 5, 10.0);

        // Wall at (8, 5) - Blocking the view
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 8, y: 5 },
            // OccupiedTiles should handle obstruction logic in the system
        ));
        let mut occupied = world.resource_mut::<crate::layer1::building::OccupiedTiles>();
        occupied.0.insert((8, 5));

        // Window at (5, 5) facing East
        world.spawn((
            Building { building_type: BuildingType::Window },
            GridPosition { x: 5, y: 5 },
            Window { direction: Direction::East, range: 10, view_cone: 0.0 },
            BeautySource { value: 0.0, radius: 2.0 },
        ));

        update_window_views_system(&mut world);

        let grid = world.resource::<BeautyGrid>();
        // View blocked by wall, beauty should be 0.0 (or very low)
        assert_eq!(grid.get(5, 5), 0.0);
    }

    #[test]
    fn test_window_negative_view() {
        let mut world = setup_world();

        // Landfill at (10, 5) - Ugly
        let mut grid = world.resource_mut::<BeautyGrid>();
        grid.set(10, 5, -10.0);

        // Window at (5, 5) facing East
        world.spawn((
            Building { building_type: BuildingType::Window },
            GridPosition { x: 5, y: 5 },
            Window { direction: Direction::East, range: 10, view_cone: 0.0 },
            BeautySource { value: 0.0, radius: 2.0 },
        ));

        update_window_views_system(&mut world);

        let grid = world.resource::<BeautyGrid>();
        // View should be negative
        assert!(grid.get(5, 5) < 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Window` Component

```rust
// src/layer1/window.rs

use bevy_ecs::prelude::*;
use crate::layer1::building::Direction;

#[derive(Component, Default)]
pub struct Window {
    pub direction: Direction,
    pub range: u32,
    pub view_cone: f32, // Angle in degrees (optional for MVP, use 0 for raycast)
}
```

### 2. Implement `update_window_views_system`

```rust
use crate::layer1::beauty::{BeautyGrid, BeautySource};
use crate::layer1::map::GridPosition;
use crate::layer1::building::{OccupiedTiles, BuildingType};

pub fn update_window_views_system(
    mut grid: ResMut<BeautyGrid>,
    occupied: Res<OccupiedTiles>, // To check walls
    windows: Query<(&GridPosition, &Window, &mut BeautySource)>,
    // Need terrain for "Natural Beauty" check? Yes.
    terrain: Res<crate::layer1::terrain::TerrainGrid>,
) {
    // 1. Snapshot the current grid state (Static Beauty) to avoid feedback loops?
    // Or just read from grid and modify it. Since windows are sparse, feedback is minimal unless two windows face each other.
    // For MVP, simple read-then-write.

    // We need to know WHICH tiles block view. Walls block.
    // We can use `BuildingType::is_obstacle` or similar?
    // Spec 004 defines `blocks_wind` which is a good proxy for "blocks view".
    // But `OccupiedTiles` just stores coordinates. We might need `BuildingMap` to lookup type.

    // For MVP Green: Just assume any Occupied Tile blocks view (except the window itself).

    for (pos, window, mut source) in &windows {
        let (dx, dy) = window.direction.to_delta();
        let mut total_view_beauty = 0.0;

        for i in 1..=window.range {
            let tx = pos.x + dx * (i as i32);
            let ty = pos.y + dy * (i as i32);

            // Bounds check
            if tx < 0 || ty < 0 || tx >= grid.width as i32 || ty >= grid.height as i32 {
                // Hit map edge. Add "Sky" bonus?
                total_view_beauty += 5.0; // Sky view bonus
                break;
            }

            // Obstruction check
            if occupied.0.contains(&(tx, ty)) {
                // Check if it's a transparent building?
                // For MVP, assume blocked.
                break;
            }

            // Read beauty at tile
            let tile_beauty = grid.get(tx as usize, ty as usize);

            // Attenuate by distance?
            // view_beauty += tile_beauty / (i as f32).sqrt();
            total_view_beauty += tile_beauty;
        }

        // Update the Window's BeautySource value
        // This will be applied to the grid in the NEXT frame's beauty update?
        // OR we apply it directly to grid now.
        // If `BeautySource` is used by `update_beauty_grid_system`, we should update the component value.
        // BUT `update_beauty_grid_system` runs before this?
        // Cycle: BeautyGrid -> WindowSystem -> BeautySource -> BeautyGrid (Next Frame).
        // This is fine. It converges.

        source.value = total_view_beauty * 0.1; // Scale factor so 100 beauty mountain doesn't explode room beauty.

        // OPTIONAL: Inject directly into grid now for instant feedback
        let current = grid.get(pos.x as usize, pos.y as usize);
        grid.set(pos.x as usize, pos.y as usize, current + source.value);
    }
}
```

### 3. Add `Window` to `BuildingType`

In `src/layer1/building.rs`:
- Add `Window` variant.
- `cost`: Glass/Wood.
- `char`: '□' or similar.
- `blocks_wind`: False (if open) or True (if glass)? Glass blocks wind but passes light. True.
- `beauty_value`: 0.0 (Calculated dynamically).

## REFACTOR Phase: Quality & Design

- **Optimization**: Raycasting every frame is costly.
  - *Cache*: Store `last_view_value`. Only recalculate if `grid_hash` changes or N ticks pass.
- **Translucency**: Allow viewing *through* other Windows or transparent structures.
- **Direction**: Infer direction from wall placement context if not explicitly set? (Auto-rotation).
- **Fog of War**: Don't reveal beauty in unrevealed tiles.

## Acceptance Criteria

- [ ] `Window` component exists.
- [ ] Windows facing "Beautiful" tiles (Statues) gain positive beauty.
- [ ] Windows facing "Ugly" tiles (Landfills) gain negative beauty.
- [ ] Walls block the view.
- [ ] Map edges (Sky) provide a bonus.
- [ ] Tests pass.

## Technical Guidance

- Use `OccupiedTiles` for obstruction checks.
- Be careful with the "Feedback Loop" of beauty. A window looking at a window looking at a statue shouldn't create infinite beauty. Using `source.value = view * 0.1` dampens it naturally.
- Ensure `update_window_views_system` runs *after* `update_beauty_grid_system` in the schedule.

## Questions

- Should "Night" affect view beauty? (Cannot see statue in dark).
- Should "Window" be a building or a modification to a Wall? (MVP: Separate Building `Window` that acts like a Wall but transparent).
