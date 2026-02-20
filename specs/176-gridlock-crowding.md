# 176 - The Gridlock (Crowding System)

**Layer:** 1
**Status:** In Progress
**Related:** `002-terrain-grid`, `016-utility-ai-system`, `121-gridlock-idea`

## 1. Overview

**Fantasy:** The claustrophobia of a busy station.

**Mechanic:** Tiles have a "Crowding" penalty. If multiple Pops occupy or traverse a tile in a short window, movement speed drops drastically (path cost increases). "Wide" corridors reduce this effect by spreading the load, while bottlenecks become congested.

**Goal:** Encourage players to build wider corridors and thoughtful layouts rather than single-tile choke points.

## 2. Dependencies

- `002-terrain-grid`: Basic grid structure.
- `004-pop-entity`: Entities that cause crowding.
- `pathfinding.rs`: Must integrate crowding cost into A*.

## 3. RED Phase: Tests First

These tests define the behavior and must be written in `src/layer1/crowding.rs` (or similar) before implementation.

```rust
#[test]
fn test_crowding_grid_initialization() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(TerrainGrid::new(10, 10));

    // Act
    // Initialize CrowdingGrid resource
    world.insert_resource(CrowdingGrid::new(10, 10));

    // Assert
    let grid = world.resource::<CrowdingGrid>();
    assert_eq!(grid.get(0, 0), 0);
}

#[test]
fn test_crowding_increases_on_traversal() {
    // Arrange
    let mut world = World::new();
    let mut grid = CrowdingGrid::new(10, 10);
    let pop_pos = GridPosition { x: 5, y: 5 };

    // Act
    // Simulate pop traversal (system update)
    grid.add_crowding(5, 5, 10); // +10 crowding

    // Assert
    assert_eq!(grid.get(5, 5), 10);
}

#[test]
fn test_crowding_decays_over_time() {
    // Arrange
    let mut grid = CrowdingGrid::new(10, 10);
    grid.add_crowding(5, 5, 10);

    // Act
    grid.decay(1); // Decay by 1 per tick

    // Assert
    assert_eq!(grid.get(5, 5), 9);
}

#[test]
fn test_pathfinding_avoids_crowded_tiles() {
    // Arrange
    let mut world = World::new();
    // Setup simple map:
    // S . . E (Path A: straight, length 3)
    // . W W .
    // . . . . (Path B: around, length 5)

    // Make Path A very crowded (Cost +100)
    let mut crowding = CrowdingGrid::new(10, 10);
    crowding.add_crowding(1, 0, 100);
    world.insert_resource(crowding);

    // Act
    let path = find_path(&world, (0, 0), (3, 0));

    // Assert
    // Path should go around the crowded tile (1,0)
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(!p.contains(&(1, 0)), "Path should avoid crowded tile (1,0)");
}
```

## 4. GREEN Phase: Minimal Implementation

### Data Structure
Create `CrowdingGrid` resource:
```rust
#[derive(Resource)]
pub struct CrowdingGrid {
    width: usize,
    height: usize,
    cells: Vec<u8>, // 0-255 crowding level
}
```

### Decay System
`crowding_decay_system`:
- Runs every tick.
- Iterates all cells.
- `cell = cell.saturating_sub(DECAY_RATE)`.

### Accumulation System
`crowding_accumulation_system`:
- Runs every tick.
- Iterates all moving Pops.
- `grid.add_crowding(pop.pos.x, pop.pos.y, CROWDING_PER_POP)`.

### Pathfinding Integration
Modify `pathfinding.rs`:
- Get `CrowdingGrid` resource.
- In `tile_cost` calculation:
  ```rust
  let crowding_cost = crowding_grid.get(next.0, next.1) as i32;
  let total_cost = terrain_cost + crowding_cost;
  ```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Use a sparse set or chunks if the map is huge (currently 100x100 is fine for dense vec).
- **Tuning**: Adjust `DECAY_RATE` (e.g., 1 per tick) and `CROWDING_PER_POP` (e.g., 10) to balance "stickiness" of traffic jams.
- **Visualization**: Add a debug view (heatmap) to seeing crowding.
- **Optimization**: Only decay active cells (store list of non-zero indices?).

## 6. Acceptance Criteria

- [ ] `CrowdingGrid` resource exists and tracks values.
- [ ] Pops increase crowding on their current tile.
- [ ] Crowding decays over time.
- [ ] Pathfinding prefers empty longer routes over crowded shorter routes.
- [ ] Test coverage > 85%.

## 7. Technical Guidance

- Use `u8` for crowding to save memory, clamping at 255.
- Decay should happen *before* accumulation in the frame to prevent immediate decay of new traffic? Or after? consistency matters.
- Pathfinding heuristic (Manhattan) might need to be weighted if crowding costs are very high, to keep A* admissible/efficient.
- Be careful with `find_path` loop—adding dynamic costs makes caching harder.

## 8. Questions

- Should stationary pops cause crowding? (Yes, standing in a hallway blocks it).
- Should crowding affect movement *speed* directly (slow down) or just pathfinding *cost* (avoidance)?
  - **Decision**: Both. Pathfinding avoids it, but if you *must* walk through, you pay the movement cost (slower).
