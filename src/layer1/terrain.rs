use super::building::{Building, BuildingType};
use super::designation::{Designation, DesignationType};
use super::needs::Needs;
use super::pop::{GridPosition, pop_display};
use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::prelude::*;
use std::collections::HashMap;

/// Represents the type of terrain in a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
}

impl TerrainType {
    /// Returns a string slice representation of the terrain.
    /// Used for rendering to avoid allocating a new String for every cell every frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    ///
    /// assert_eq!(TerrainType::Grass.as_str(), ".");
    /// assert_eq!(TerrainType::Rock.as_str(), "#");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grass => ".",
            Self::Dirt => ",",
            Self::Rock => "#",
            Self::Water => "~",
            Self::Tree => "↑",
        }
    }

    /// Returns the color associated with this terrain type.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(TerrainType::Grass.color(), Color::Green);
    /// ```
    #[must_use]
    pub const fn color(self) -> Color {
        match self {
            Self::Grass => Color::Green,
            Self::Dirt => Color::Rgb(139, 90, 43),
            Self::Rock => Color::DarkGray,
            Self::Water => Color::Blue,
            Self::Tree => Color::Rgb(0, 100, 0),
        }
    }

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
            Some(self.tiles[y * self.width + x])
        } else {
            None
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

/// Cache for renderable entities to avoid repeated allocations and iterations.
#[derive(Resource, Default)]
pub struct RenderCache {
    /// Cached pop display data.
    pub pops: HashMap<GridPosition, (&'static str, Color)>,
    /// Cached building data.
    pub buildings: HashMap<GridPosition, BuildingType>,
    /// Cached designation data.
    pub designations: HashMap<GridPosition, DesignationType>,
}

/// Updates the `RenderCache` by iterating the world once.
pub fn update_render_cache(world: &mut World) {
    let mut cache = world.remove_resource::<RenderCache>().unwrap_or_default();

    cache.pops.clear();
    cache.buildings.clear();
    cache.designations.clear();

    for e in world.iter_entities() {
        if let Some(pos) = e.get::<GridPosition>() {
            // Check for Pop (via Needs)
            if let Some(needs) = e.get::<Needs>() {
                cache.pops.insert(*pos, pop_display(needs));
            }

            // Check for Building
            if let Some(building) = e.get::<Building>() {
                cache.buildings.insert(*pos, building.building_type);
            }

            // Check for Designation
            if let Some(designation) = e.get::<Designation>() {
                cache.designations.insert(*pos, designation.designation_type);
            }
        }
    }

    world.insert_resource(cache);
}

/// Generates a new terrain grid with procedural features.
#[must_use]
pub fn generate_terrain(width: usize, height: usize) -> TerrainGrid {
    let mut rng = rand::thread_rng();
    let mut tiles = vec![TerrainType::Grass; width * height];

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
    let r2 = (radius * radius) as i32;
    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            if dx * dx + dy * dy <= r2 {
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    tiles[y as usize * width + x as usize] = terrain_type;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_type_colors() {
        // Test color invariants for all terrain types
        assert_eq!(TerrainType::Grass.color(), Color::Green);
        assert_eq!(TerrainType::Dirt.color(), Color::Rgb(139, 90, 43));
        assert_eq!(TerrainType::Rock.color(), Color::DarkGray);
        assert_eq!(TerrainType::Water.color(), Color::Blue);
    }

    #[test]
    fn test_terrain_type_as_str() {
        // Test string representation invariants
        assert_eq!(TerrainType::Grass.as_str(), ".");
        assert_eq!(TerrainType::Dirt.as_str(), ",");
        assert_eq!(TerrainType::Rock.as_str(), "#");
        assert_eq!(TerrainType::Water.as_str(), "~");
    }

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
        // Test new variant properties
        assert_eq!(TerrainType::Tree.as_str(), "↑");
        assert_eq!(TerrainType::Tree.color(), Color::Rgb(0, 100, 0));
        assert_eq!(TerrainType::Tree.name(), "Tree");
    }

    #[test]
    fn test_render_cache_updates() {
        use crate::layer1::pop::Pop;

        let mut world = World::new();
        world.insert_resource(RenderCache::default());

        // Spawn a pop
        world.spawn((
            Pop,
            GridPosition { x: 1, y: 1 },
            Needs::default(),
        ));

        // Spawn a building
        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 2, y: 2 },
        ));

        // Spawn a designation
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 3, y: 3 },
        ));

        update_render_cache(&mut world);

        let cache = world.resource::<RenderCache>();
        assert!(cache.pops.contains_key(&GridPosition { x: 1, y: 1 }));
        assert!(cache.buildings.contains_key(&GridPosition { x: 2, y: 2 }));
        assert!(cache.designations.contains_key(&GridPosition { x: 3, y: 3 }));
        assert_eq!(cache.pops.len(), 1);
        assert_eq!(cache.buildings.len(), 1);
        assert_eq!(cache.designations.len(), 1);
    }

    #[test]
    fn test_render_cache_clears_old_data() {
        use crate::layer1::pop::Pop;

        let mut world = World::new();
        world.insert_resource(RenderCache::default());

        let entity = world
            .spawn((Pop, GridPosition { x: 1, y: 1 }, Needs::default()))
            .id();

        update_render_cache(&mut world);
        assert_eq!(world.resource::<RenderCache>().pops.len(), 1);

        // Despawn and update
        world.despawn(entity);
        update_render_cache(&mut world);
        assert_eq!(world.resource::<RenderCache>().pops.len(), 0);
    }
}
