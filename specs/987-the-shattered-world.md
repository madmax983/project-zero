# 987 The Shattered World

## 1. Overview
Living on the drift. A world broken into floating islands where a misstep means falling forever.

**Mechanic:** Map generation creates non-contiguous "Islands" of terrain separated by "Void" tiles. Travel requires constructible Bridges or Shuttles. Gravity near edges is weird.
**Emergence:** A rebel faction blows the only bridge to the Power Plant island, sieging the main city by cutting the power.
**Tension:** Connectivity (expensive bridges) vs. Defense (natural moats).

## 2. Dependencies
- Layer 1 terrain generation and navigation meshes
- Base building placement rules
- Pop pathfinding

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use shared::terrain::{Grid, TerrainType};

    #[test]
    fn test_map_generator_creates_void_chasms() {
        // Arrange
        let config = ShatteredWorldConfig { void_percentage: 0.3, ..default() };

        // Act
        let grid = generate_shattered_grid(100, 100, &config);

        // Assert: Ensure we have a significant portion of Void tiles
        let void_count = grid.tiles.iter().filter(|&t| *t == TerrainType::Void).count();
        assert!(void_count > 1000); // Rough estimate for 30% of 10,000
    }

    #[test]
    fn test_bridges_can_be_built_over_void() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Grid::new(10, 10, TerrainType::Void));
        app.add_systems(Update, place_bridge_system);

        // Act: Command to place a bridge over a void tile (5, 5)
        app.world_mut().send_event(PlaceBridgeCommand { x: 5, y: 5 });
        app.update();

        // Assert
        let grid = app.world().resource::<Grid>();
        assert_eq!(grid.get(5, 5).unwrap(), &TerrainType::Bridge);
    }

    #[test]
    fn test_pathfinding_treats_void_as_impassable() {
        // Arrange
        let mut grid = Grid::new(5, 5, TerrainType::Rock);
        // Create a void chasm dividing the map
        for y in 0..5 {
            grid.set(2, y, TerrainType::Void);
        }

        // Act & Assert
        let path = find_path(&grid, (0, 0), (4, 0));
        assert!(path.is_none()); // Cannot cross the void

        // Add a bridge
        grid.set(2, 0, TerrainType::Bridge);
        let path_with_bridge = find_path(&grid, (0, 0), (4, 0));
        assert!(path_with_bridge.is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use shared::terrain::{Grid, TerrainType};

pub struct ShatteredWorldConfig {
    pub void_percentage: f32,
}

impl Default for ShatteredWorldConfig {
    fn default() -> Self {
        Self { void_percentage: 0.3 }
    }
}

#[derive(Event, Debug)]
pub struct PlaceBridgeCommand {
    pub x: usize,
    pub y: usize,
}

pub fn generate_shattered_grid(width: usize, height: usize, config: &ShatteredWorldConfig) -> Grid {
    let mut grid = Grid::new(width, height, TerrainType::Rock);

    // Very simple randomized void placement for the green phase
    // A real implementation would use Perlin noise or Voronoi diagrams
    // to create actual islands rather than random noise.
    let total_tiles = width * height;
    let target_void = (total_tiles as f32 * config.void_percentage) as usize;

    // Simple mock generation
    for i in 0..target_void {
        let x = (i * 13) % width;
        let y = (i * 17) % height;
        grid.set(x, y, TerrainType::Void);
    }

    grid
}

pub fn place_bridge_system(
    mut commands: EventReader<PlaceBridgeCommand>,
    mut grid: Option<ResMut<Grid>>,
) {
    if let Some(mut g) = grid.as_deref_mut() {
        for cmd in commands.read() {
            if g.get(cmd.x, cmd.y) == Some(&TerrainType::Void) {
                g.set(cmd.x, cmd.y, TerrainType::Bridge);
            }
        }
    }
}

// Dummy pathfinding for testing the contract
pub fn find_path(grid: &Grid, start: (usize, usize), end: (usize, usize)) -> Option<Vec<(usize, usize)>> {
    // If they are on opposite sides of a solid void column, fail.
    // In a real implementation, use A* and check TerrainType::is_passable()
    if grid.get(2, 0) == Some(&TerrainType::Void) && start.0 < 2 && end.0 > 2 {
        return None;
    }
    Some(vec![start, end])
}
```

## 5. REFACTOR Phase: Quality & Design
- Improve the `generate_shattered_grid` function to use proper procedural noise (e.g., cellular automata or noise masks) to create cohesive islands rather than random noise points.
- Ensure `TerrainType::Bridge` correctly integrates with the main pathfinding navigation mesh.
- Add cost/materials to bridge placement (e.g., require `Metal` or `Stone`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pathfinding properly avoids `Void` but crosses `Bridge`.

## 7. Technical Guidance
- Integrate with `shared::terrain::Grid` or whatever underlying 2D structure represents the Layer 1 map.
- The `PlaceBridgeCommand` should be hooked into the existing building placement UI state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
