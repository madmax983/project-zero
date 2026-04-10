use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Usage threshold to convert Grass to Dirt.
pub const EROSION_THRESHOLD_DIRT: u16 = 100;
/// Usage threshold to convert Dirt to Path.
pub const EROSION_THRESHOLD_PATH: u16 = 500;
/// Usage threshold to revert Path to Dirt (Hysteresis).
pub const RECOVERY_THRESHOLD_PATH: u16 = 400;
/// Usage threshold to revert Dirt to Grass (Hysteresis).
pub const RECOVERY_THRESHOLD_DIRT: u16 = 50;
/// Amount of usage added per step.
pub const MOVEMENT_EROSION_AMOUNT: u16 = 5;

/// Grid tracking usage/erosion on each tile.
#[derive(Resource, Default)]
pub struct ErosionGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Usage counters.
    pub values: Vec<u16>,
}

impl ErosionGrid {
    /// Create a new erosion grid.
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

    /// Add erosion usage to a tile.
    pub fn add_erosion(&mut self, x: usize, y: usize, amount: u16) {
        if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
            if idx < self.values.len() {
                self.values[idx] = self.values[idx].saturating_add(amount);
            }
        }
    }

    /// Get usage value at a tile.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> u16 {
        if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
            if idx < self.values.len() {
                return self.values[idx];
            }
        }
        0
    }

    /// Set usage value at a tile.
    pub fn set(&mut self, x: usize, y: usize, val: u16) {
        if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
            if idx < self.values.len() {
                self.values[idx] = val;
            }
        }
    }
}

/// System that converts terrain based on accumulated erosion usage.
pub fn update_erosion_system(mut terrain: ResMut<TerrainGrid>, erosion: Res<ErosionGrid>) {
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            let Some(idx) = y.checked_mul(terrain.width).and_then(|i| i.checked_add(x)) else {
                continue;
            };
            if idx >= terrain.tiles.len() || idx >= erosion.values.len() {
                continue;
            }

            let current_type = terrain.tiles[idx];
            let erosion_val = erosion.values[idx];

            if current_type == TerrainType::Grass && erosion_val >= EROSION_THRESHOLD_DIRT {
                terrain.tiles[idx] = TerrainType::Dirt;
            } else if current_type == TerrainType::Dirt && erosion_val >= EROSION_THRESHOLD_PATH {
                terrain.tiles[idx] = TerrainType::Path;
            }
        }
    }
}

/// System that decays erosion usage and reverts terrain (Path -> Dirt -> Grass).
pub fn regrowth_system(mut terrain: ResMut<TerrainGrid>, mut erosion: ResMut<ErosionGrid>) {
    for i in 0..erosion.values.len() {
        if erosion.values[i] > 0 {
            erosion.values[i] = erosion.values[i].saturating_sub(1);
        }

        if i < terrain.tiles.len() {
            let current_type = terrain.tiles[i];
            let val = erosion.values[i];

            if current_type == TerrainType::Path && val < RECOVERY_THRESHOLD_PATH {
                terrain.tiles[i] = TerrainType::Dirt;
            } else if current_type == TerrainType::Dirt && val < RECOVERY_THRESHOLD_DIRT {
                terrain.tiles[i] = TerrainType::Grass;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::erosion::{update_erosion_system, ErosionGrid};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        let width: usize = 10;
        let height = 10;

        // Setup Terrain
        let size = width.checked_mul(height).expect("Grid size overflow");
        assert!(size <= 10_000_000, "Grid size too large");
        let terrain = TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; size],
        };
        world.insert_resource(terrain);

        // Setup Erosion Grid
        world.insert_resource(ErosionGrid::new(width, height));

        world
    }

    #[test]
    fn test_erosion_accumulation() {
        let mut world = setup_world();
        let mut erosion_grid = world.resource_mut::<ErosionGrid>();

        // Simulate step
        erosion_grid.add_erosion(5, 5, 10);
        assert_eq!(erosion_grid.get(5, 5), 10);
    }

    #[test]
    fn test_grass_erodes_to_dirt() {
        let mut world = setup_world();
        let mut erosion_grid = world.resource_mut::<ErosionGrid>();

        // Set erosion just below threshold
        erosion_grid.set(5, 5, 99);

        // Add more erosion to cross threshold (assumed 100)
        erosion_grid.add_erosion(5, 5, 2);

        // Run update system
        world.run_system_once(update_erosion_system).unwrap();

        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));
    }

    #[test]
    fn test_dirt_erodes_to_path() {
        let mut world = setup_world();
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            // Start as Dirt
            terrain.tiles[55] = TerrainType::Dirt; // (5,5) assuming 10 width

            let mut erosion_grid = world.resource_mut::<ErosionGrid>();
            // Set high erosion (assumed threshold 500)
            erosion_grid.set(5, 5, 501);
        }

        world.run_system_once(update_erosion_system).unwrap();

        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Path));
    }

    #[test]
    fn test_path_decay_unused() {
        let mut world = setup_world();
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Path;

            let mut erosion_grid = world.resource_mut::<ErosionGrid>();
            erosion_grid.set(5, 5, 0); // No usage
        }

        // Simulate many ticks of regrowth
        for _ in 0..100 {
            world.run_system_once(super::regrowth_system).unwrap();
        }

        let terrain = world.resource::<TerrainGrid>();
        // Should revert to Dirt or Grass
        assert_ne!(terrain.get(5, 5), Some(TerrainType::Path));
    }

    #[test]
    fn test_path_speed_bonus() {
        // This likely belongs in movement_tests.rs or similar, but spec defines it.
        // Assuming movement cost function exists:
        let cost_grass = TerrainType::Grass.movement_cost();
        let cost_path = TerrainType::Path.movement_cost();

        assert!(cost_path < cost_grass, "Path should be faster than Grass");
    }
}
