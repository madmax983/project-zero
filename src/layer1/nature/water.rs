//! Water simulation logic (Spec 096).

use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Maximum hydration value for a tile.
pub const MAX_HYDRATION: u8 = 100;
/// Hydration decay per tile distance.
pub const HYDRATION_DECAY: u8 = 10; // Drop per tile distance

/// Grid representing water hydration levels across the map.
#[derive(Resource, Default)]
pub struct WaterGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Flattened hydration values (0-100).
    pub values: Vec<u8>,
}

impl WaterGrid {
    /// Creates a new `WaterGrid` with the specified dimensions.
    ///
    /// # Panics
    /// Panics if `width.checked_mul(height).expect("overflow")` overflows or exceeds 10_000_000.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");

        Self {
            width,
            height,
            values: vec![0; size],
        }
    }

    /// Gets the hydration value at the specified coordinates.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.values[y * self.width + x]
    }

    /// Sets the hydration value at the specified coordinates.
    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[y * self.width + x] = value;
    }
}

/// Component for entities that act as a source of water (e.g., Wells).
#[derive(Component)]
pub struct WaterSource {
    /// Range of the water source (not currently used in simple diffusion).
    pub range: u32,
    /// Amount of hydration this source provides to its tile.
    pub amount: u8,
}

/// System to update water hydration levels based on terrain and sources.
///
/// Uses a double-pass diffusion algorithm to spread water from sources.
pub fn update_water_system(
    mut water: ResMut<WaterGrid>,
    terrain: Res<TerrainGrid>,
    sources: Query<(&GridPosition, &WaterSource)>,
) {
    let width = water.width;
    let height = water.height;
    // Reset grid to 0 every tick to allow water to recede
    // Bolt: Reuse existing buffer instead of allocating new Vec every tick
    water.values.fill(0);

    // 1. Set sources (Terrain::Water)
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if terrain.tiles[idx] == TerrainType::Water {
                water.values[idx] = MAX_HYDRATION;
            }
        }
    }

    // Set sources (Buildings)
    for (pos, source) in &sources {
        if pos.x >= 0 && pos.y >= 0 {
            #[allow(clippy::cast_sign_loss)]
            let x = pos.x as usize;
            #[allow(clippy::cast_sign_loss)]
            let y = pos.y as usize;
            if x < width && y < height {
                let idx = y * width + x;
                water.values[idx] = source.amount;
            }
        }
    }

    // 2. Diffusion (Simple 4-way spread)

    // Pass 1: Top-Left -> Bottom-Right
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if terrain.tiles[idx] == TerrainType::Rock {
                continue;
            }

            let mut max_neighbor = 0;

            // Check Left
            if x > 0 {
                max_neighbor = max_neighbor.max(water.values[idx - 1]);
            }
            // Check Top
            if y > 0 {
                max_neighbor = max_neighbor.max(water.values[idx - width]);
            }

            let potential = max_neighbor.saturating_sub(HYDRATION_DECAY);
            if potential > water.values[idx] {
                water.values[idx] = potential;
            }
        }
    }

    // Pass 2: Bottom-Right -> Top-Left
    for y in (0..height).rev() {
        for x in (0..width).rev() {
            let idx = y * width + x;
            if terrain.tiles[idx] == TerrainType::Rock {
                continue;
            }

            let mut max_neighbor = 0;

            // Check Right
            if x < width - 1 {
                max_neighbor = max_neighbor.max(water.values[idx + 1]);
            }
            // Check Bottom
            if y < height - 1 {
                max_neighbor = max_neighbor.max(water.values[idx + width]);
            }

            let potential = max_neighbor.saturating_sub(HYDRATION_DECAY);
            if potential > water.values[idx] {
                water.values[idx] = potential;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world(width: usize, height: usize) -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; width.checked_mul(height).expect("overflow")];
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles,
        });
        world.insert_resource(WaterGrid::new(width, height));
        world
    }

    #[test]
    fn test_water_grid_initialization() {
        let world = setup_world(10, 10);
        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert!(grid.values.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_terrain_water_is_infinite_source() {
        let mut world = setup_world(10, 10);

        // Set (5,5) to Water terrain
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Water; // (5, 5)
        }

        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        assert_eq!(
            grid.get(5, 5),
            MAX_HYDRATION,
            "Water terrain should be fully hydrated"
        );
    }

    #[test]
    fn test_water_spreads_to_neighbors() {
        let mut world = setup_world(10, 10);

        // Set (5,5) to Water
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Water;
        }

        // Run once
        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        // Neighbors should have water (slightly less than max)
        assert!(grid.get(5, 4) > 0, "North should get water");
        assert!(grid.get(5, 6) > 0, "South should get water");
        assert!(grid.get(4, 5) > 0, "West should get water");
        assert!(grid.get(6, 5) > 0, "East should get water");

        assert!(grid.get(5, 4) < MAX_HYDRATION, "Spread water should decay");
    }

    #[test]
    fn test_water_source_building() {
        let mut world = setup_world(10, 10);

        // Spawn a Well (WaterSource) at (5,5)
        // Use BuildingType::Well which now spawns WaterSource automatically
        // But for this test, we might want to manually spawn it to isolate from spawn_building logic?
        // Or we can rely on spawn_building (if imported).
        // Since spawn_building is in layer1::building, and we are testing layer1::water,
        // we can just stick to manual spawning or use the placeholder if Well doesn't exist yet (it does).

        // Let's manually spawn components as per original test intent, but use Well type.
        world.spawn((
            Building {
                building_type: BuildingType::Well,
            },
            GridPosition { x: 5, y: 5 },
            WaterSource {
                range: 5,
                amount: MAX_HYDRATION,
            },
        ));

        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        assert_eq!(
            grid.get(5, 5),
            MAX_HYDRATION,
            "Source should hydrate its tile"
        );
        assert!(grid.get(6, 5) > 0, "Source should spread water");
    }

    #[test]
    fn test_rock_blocks_water() {
        let mut world = setup_world(10, 10);

        // (5,5) Water. Column 6 is Rock (Blocking Wall).
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Water;
            for y in 0..10 {
                terrain.tiles[y * 10 + 6] = TerrainType::Rock;
            }
        }

        // Run multiple ticks to allow spread
        for _ in 0..5 {
            world.run_system_once(update_water_system).unwrap();
        }

        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.get(5, 5), MAX_HYDRATION);
        assert_eq!(grid.get(6, 5), 0, "Rock should not hold water");
        assert_eq!(grid.get(7, 5), 0, "Water should not pass through Rock Wall");
    }

    #[test]
    fn test_water_recedes_when_source_removed() {
        let mut world = setup_world(10, 10);

        // Spawn a Well
        let well = world
            .spawn((
                Building {
                    building_type: BuildingType::Well,
                },
                GridPosition { x: 5, y: 5 },
                WaterSource {
                    range: 5,
                    amount: MAX_HYDRATION,
                },
            ))
            .id();

        // Tick to spread water
        world.run_system_once(update_water_system).unwrap();
        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.get(5, 5), MAX_HYDRATION);

        // Remove Well
        world.despawn(well);

        // Tick again - water should disappear
        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        assert_eq!(
            grid.get(5, 5),
            0,
            "Water should recede instantly if source removed"
        );
    }
}
