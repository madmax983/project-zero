use bevy_ecs::prelude::*;
use rand::Rng;

/// Represents the type of terrain in a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TerrainType {
    /// Green grass, the default ground.
    Grass,
    /// Brown dirt, often found in patches.
    Dirt,
    /// Grey rock, harder material.
    Rock,
    /// Blue water, impassable by normal means.
    Water,
    /// Green tree, yields wood when chopped.
    Tree,
    /// Heavily trodden path, faster movement but lower beauty.
    Path,
    /// Pioneer vegetation, the first stage of succession.
    Shrub,
    /// Young tree, will grow into a full tree.
    Sapling,
}

impl TerrainType {
    /// Returns the human-readable name of the terrain type.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    ///
    /// assert_eq!(TerrainType::Grass.name(), "Grass");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Dirt => "Dirt",
            Self::Rock => "Rock",
            Self::Water => "Water",
            Self::Tree => "Tree",
            Self::Path => "Path",
            Self::Shrub => "Shrub",
            Self::Sapling => "Sapling",
        }
    }

    /// Returns the movement cost for this terrain type.
    ///
    /// Lower is faster. Default is 1.0.
    #[must_use]
    pub const fn movement_cost(self) -> f32 {
        match self {
            Self::Path => 0.8,
            Self::Tree => 1.5,
            Self::Shrub => 1.2,
            Self::Sapling => 1.1,
            _ => 1.0,
        }
    }

    /// Returns whether this terrain type is walkable by pops.
    ///
    /// Rock and Water are not walkable.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    ///
    /// assert!(TerrainType::Grass.is_walkable());
    /// assert!(!TerrainType::Rock.is_walkable());
    /// assert!(!TerrainType::Water.is_walkable());
    /// ```
    #[must_use]
    pub const fn is_walkable(self) -> bool {
        !matches!(self, Self::Rock | Self::Water)
    }

    /// Returns the thermal retention (0.0 to 1.0) of the terrain (Spec 198).
    /// - 0.5: Rock (Thermal Mass)
    /// - 0.2: Water (Specific Heat)
    /// - 0.1: Grass/Dirt
    #[must_use]
    pub const fn heat_retention(self) -> f32 {
        match self {
            Self::Rock => 0.5,
            Self::Water => 0.2,
            Self::Grass | Self::Dirt | Self::Path | Self::Tree | Self::Shrub | Self::Sapling => 0.1,
        }
    }
}

/// A 2D grid representing the game map's terrain layer.
#[derive(Resource)]
pub struct TerrainGrid {
    /// The width of the grid in cells.
    pub width: usize,
    /// The height of the grid in cells.
    pub height: usize,
    /// The flat vector of terrain tiles, stored in row-major order.
    pub tiles: Vec<TerrainType>, // row-major: index = y * width + x
}

impl TerrainGrid {
    /// Retrieves the terrain type at the specified coordinates, if within bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::{TerrainGrid, TerrainType, generate_terrain};
    ///
    /// let grid = generate_terrain(10, 10);
    /// if let Some(tile) = grid.get(0, 0) {
    ///     println!("Tile at (0,0) is {:?}", tile);
    /// }
    /// assert!(grid.get(100, 100).is_none());
    /// ```
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            // Use checked arithmetic to prevent overflow wrapping around to a valid index
            // and verify the index is within the actual data buffer.
            let idx = y.checked_mul(self.width)?.checked_add(x)?;
            if idx < self.tiles.len() {
                Some(self.tiles[idx])
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Sets the terrain type at the specified coordinates.
    pub fn set(&mut self, x: usize, y: usize, tile: TerrainType) {
        if x < self.width && y < self.height {
            if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
                if idx < self.tiles.len() {
                    self.tiles[idx] = tile;
                }
            }
        }
    }
}

/// Defines the visible area of the map for the player.
#[derive(Resource, Default, Clone, Copy)]
pub struct Viewport {
    /// The x-coordinate of the top-left corner of the viewport in grid space.
    pub x: i32,
    /// The y-coordinate of the top-left corner of the viewport in grid space.
    pub y: i32,
}

/// Generates a new terrain grid with procedural features.
///
/// # Panics
///
/// Panics if `width` or `height` is zero.
#[must_use]
pub fn generate_terrain(width: usize, height: usize) -> TerrainGrid {
    assert!(
        width > 0 && height > 0,
        "Terrain dimensions must be positive"
    );
    let count = width.checked_mul(height).expect("Terrain size overflow");
    assert!(count <= 1_000_000, "Terrain too large (max 1M tiles)");

    let mut rng = rand::thread_rng();
    let mut tiles = vec![TerrainType::Grass; count];

    // Scatter some dirt patches
    for _ in 0..50 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(2..6);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Dirt);
    }

    // Scatter some rock
    for _ in 0..30 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(1..4);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Rock);
    }

    // Scatter some trees (forests)
    for _ in 0..40 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(2..5);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Tree);
    }

    // A river or lake
    for _ in 0..10 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(3..8);
        fill_circle(
            &mut tiles,
            width,
            height,
            cx,
            cy,
            radius,
            TerrainType::Water,
        );
    }

    TerrainGrid {
        width,
        height,
        tiles,
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn fill_circle(
    tiles: &mut [TerrainType],
    width: usize,
    height: usize,
    cx: usize,
    cy: usize,
    radius: usize,
    terrain_type: TerrainType,
) {
    // Clamp radius to avoid overflow (sqrt(i32::MAX) ~= 46340)
    let safe_radius = radius.min(46_000);
    let r_i32 = i32::try_from(safe_radius).unwrap_or(i32::MAX);
    let r2 = r_i32.saturating_mul(r_i32);

    let max_x = i32::try_from(width).unwrap_or(i32::MAX);
    let max_y = i32::try_from(height).unwrap_or(i32::MAX);
    let cx_i32 = i32::try_from(cx).unwrap_or(i32::MAX);
    let cy_i32 = i32::try_from(cy).unwrap_or(i32::MAX);

    for dy in -r_i32..=r_i32 {
        for dx in -r_i32..=r_i32 {
            if dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)) <= r2 {
                let x = cx_i32.saturating_add(dx);
                let y = cy_i32.saturating_add(dy);
                if x >= 0 && x < max_x && y >= 0 && y < max_y {
                    let idx = (y as usize)
                        .saturating_mul(width)
                        .saturating_add(x as usize);
                    if idx < tiles.len() {
                        tiles[idx] = terrain_type;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generation() {
        let grid = generate_terrain(80, 50);
        assert_eq!(grid.width, 80);
        assert_eq!(grid.height, 50);
        assert_eq!(grid.tiles.len(), 80 * 50);

        // Check that the grid contains grass and only valid terrain types
        let has_grass = grid.tiles.contains(&TerrainType::Grass);
        assert!(has_grass, "Generated terrain should include grass");

        let all_valid = grid.tiles.iter().all(|t| {
            matches!(
                t,
                TerrainType::Grass
                    | TerrainType::Dirt
                    | TerrainType::Rock
                    | TerrainType::Water
                    | TerrainType::Tree
            )
        });
        assert!(all_valid, "All tiles must be valid terrain types");
    }

    #[test]
    fn test_terrain_grid_get_bounds() {
        let width = 10;
        let height = 5;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // Valid access
        assert_eq!(grid.get(0, 0), Some(TerrainType::Grass));
        assert_eq!(grid.get(width - 1, height - 1), Some(TerrainType::Grass));

        // Out of bounds
        assert_eq!(grid.get(width, 0), None);
        assert_eq!(grid.get(0, height), None);
        assert_eq!(grid.get(width, height), None);
        assert_eq!(grid.get(100, 100), None);
    }

    #[test]
    fn test_fill_circle_clipping() {
        let width = 10;
        let height = 10;
        let mut tiles = vec![TerrainType::Grass; width * height];

        // Draw a circle of Dirt at (0,0) with radius 2.
        // Should cover (0,0), (0,1), (0,2), (1,0), (1,1), (2,0) etc.
        // and safely ignore negative coordinates.
        fill_circle(&mut tiles, width, height, 0, 0, 2, TerrainType::Dirt);

        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // (0,0) should be Dirt
        assert_eq!(grid.get(0, 0), Some(TerrainType::Dirt));
        // (2,0) should be Dirt (distance 2 <= 2)
        assert_eq!(grid.get(2, 0), Some(TerrainType::Dirt));
        // (0,2) should be Dirt
        assert_eq!(grid.get(0, 2), Some(TerrainType::Dirt));
        // (3,0) should be Grass (distance 3 > 2)
        assert_eq!(grid.get(3, 0), Some(TerrainType::Grass));

        // Verify no panic or wrapping happened (check last element is still grass if far away)
        assert_eq!(grid.get(9, 9), Some(TerrainType::Grass));
    }

    #[test]
    fn test_fill_circle_all_terrain_types() {
        let width = 10;
        let height = 10;

        for terrain_type in [
            TerrainType::Grass,
            TerrainType::Dirt,
            TerrainType::Rock,
            TerrainType::Water,
        ] {
            let mut tiles = vec![TerrainType::Grass; width * height];
            fill_circle(&mut tiles, width, height, 5, 5, 2, terrain_type);

            // Center should be the terrain type
            assert_eq!(tiles[5 * width + 5], terrain_type);
        }
    }

    #[test]
    fn test_terrain_type_tree() {
        assert_eq!(TerrainType::Tree.name(), "Tree");
    }

    #[test]
    fn test_terrain_type_is_walkable() {
        assert!(TerrainType::Grass.is_walkable());
        assert!(TerrainType::Dirt.is_walkable());
        assert!(TerrainType::Tree.is_walkable());
        assert!(TerrainType::Path.is_walkable());
        assert!(!TerrainType::Rock.is_walkable());
        assert!(!TerrainType::Water.is_walkable());
    }

    #[test]
    fn test_movement_cost() {
        assert!((TerrainType::Path.movement_cost() - 0.8).abs() < f32::EPSILON);
        assert!((TerrainType::Grass.movement_cost() - 1.0).abs() < f32::EPSILON);
        assert!((TerrainType::Tree.movement_cost() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    #[should_panic(expected = "Terrain dimensions must be positive")]
    fn test_generate_terrain_zero_size_panics() {
        let _ = generate_terrain(0, 10);
    }

    #[test]
    #[should_panic(expected = "Terrain too large")]
    fn test_generate_terrain_too_large_panics() {
        let _ = generate_terrain(1001, 1000); // 1,001,000 tiles
    }

    #[test]
    fn test_get_overflow_protection() {
        // Construct a grid with huge dimensions but small buffer
        // This simulates a potentially malicious or corrupted state
        let width = usize::MAX / 2;
        let height = 10;
        let tiles = vec![TerrainType::Grass; 1];

        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // (2, 0) -> index 2. 2 > 1. Should be None.
        assert_eq!(grid.get(2, 0), None);

        // (0, 0) -> index 0. 0 < 1. Should be Some.
        assert_eq!(grid.get(0, 0), Some(TerrainType::Grass));
    }
}
