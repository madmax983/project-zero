use bevy_ecs::prelude::*;

/// Amount of crowding added per pop per tick (or per movement).
const CROWDING_PER_POP: u8 = 10;
/// Amount of crowding that decays per tick.
const DECAY_RATE: u8 = 1;

/// A grid tracking the "crowdedness" of each tile.
/// High values indicate traffic jams.
#[derive(Resource)]
pub struct CrowdingGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Crowding values (0-255).
    pub cells: Vec<u8>,
}

impl CrowdingGrid {
    /// Creates a new, empty `CrowdingGrid`.
    ///
    /// # Panics
    /// Panics if `width * height` overflows or exceeds 10,000,000.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");

        Self {
            width,
            height,
            cells: vec![0; size],
        }
    }

    /// Gets the crowding level at the specified coordinates.
    /// Returns 0 if out of bounds.
    #[must_use]
    #[allow(clippy::collapsible_if)]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            // Safe index calculation
            if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
                if idx < self.cells.len() {
                    return self.cells[idx];
                }
            }
        }
        0
    }

    /// Adds crowding to a specific tile.
    #[allow(clippy::collapsible_if)]
    pub fn add_crowding(&mut self, x: usize, y: usize, amount: u8) {
        if x < self.width && y < self.height {
            if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
                if idx < self.cells.len() {
                    self.cells[idx] = self.cells[idx].saturating_add(amount);
                }
            }
        }
    }

    /// Decays crowding across the entire grid.
    pub fn decay(&mut self, amount: u8) {
        for cell in &mut self.cells {
            *cell = cell.saturating_sub(amount);
        }
    }
}

/// System to decay crowding over time.
pub fn crowding_decay_system(mut grid: ResMut<CrowdingGrid>) {
    grid.decay(DECAY_RATE);
}

/// System to add crowding based on pop positions.
pub fn crowding_accumulation_system(
    mut grid: ResMut<CrowdingGrid>,
    query: Query<&crate::layer1::map::GridPosition, With<crate::layer1::pop::Pop>>,
) {
    for pos in &query {
        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            grid.add_crowding(x, y, CROWDING_PER_POP);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pathfinding::find_path;
    use crate::layer1::terrain::TerrainGrid;
    use crate::layer1::terrain::TerrainType;

    #[test]
    fn test_crowding_grid_initialization() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

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
        let mut _world = World::new();
        let mut grid = CrowdingGrid::new(10, 10);
        let _pop_pos = GridPosition { x: 5, y: 5 };

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

        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        world.insert_resource(crate::layer1::building::BuildingMap::default());

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
        // If pathfinding ignores crowding, it will be (0,0)->(1,0)->(2,0)->(3,0)
        // If it respects crowding, it should go (0,0)->(0,1)->(1,1)->(2,1)->(3,1)->(3,0) or similar.
        // The crowded tile is (1,0).
        assert!(!p.contains(&(1, 0)), "Path should avoid crowded tile (1,0)");
    }
}
