use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

/// Configuration for ecological succession rates.
#[derive(Resource)]
pub struct EcologyConfig {
    /// Fraction of map tiles to check per tick (0.0 to 1.0).
    pub growth_rate: f32,
    /// Probability that Dirt becomes Grass.
    pub pioneer_chance: f32,
    /// Probability that Grass becomes a Sapling (if near a Tree).
    pub seed_spread_chance: f32,
    /// Probability that a Sapling becomes a Tree.
    pub maturation_chance: f32,
}

impl Default for EcologyConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.05,
            pioneer_chance: 0.1,
            seed_spread_chance: 0.05,
            maturation_chance: 0.02,
        }
    }
}

/// Processes ecological succession for the terrain.
///
/// This system simulates natural vegetation growth:
/// - Dirt -> Grass (Pioneer species)
/// - Grass -> Sapling (Seeding from nearby trees)
/// - Grass -> Shrub (Random growth)
/// - Sapling -> Tree (Maturation)
pub fn process_ecological_succession(world: &mut World) {
    let config = world.resource::<EcologyConfig>();
    // We need config values to avoid borrow checker issues when borrowing world mutably later
    let growth_rate = config.growth_rate;
    let pioneer_chance = f64::from(config.pioneer_chance);
    let seed_spread_chance = f64::from(config.seed_spread_chance);
    let maturation_chance = f64::from(config.maturation_chance);
    let shrub_chance = pioneer_chance * 0.5;

    // Use a scoped block to mutate grid
    let mut grid = world.resource_mut::<TerrainGrid>();
    let width = grid.width;
    let height = grid.height;

    // Calculate number of tiles to update based on growth_rate
    let total_tiles = width * height;
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let tiles_to_update = (total_tiles as f32 * growth_rate).ceil() as usize;

    let mut rng = rand::thread_rng();

    for _ in 0..tiles_to_update {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);

        // We can't easily check neighbors while holding a mutable reference to the grid if we use `get` and `set`.
        // However, `grid.get` takes `&self` and `grid.set` takes `&mut self`.
        // We can clone the current tile type first.
        let Some(current) = grid.get(x, y) else {
            continue;
        };

        let new_type = match current {
            TerrainType::Dirt => {
                if rng.gen_bool(pioneer_chance) {
                    Some(TerrainType::Grass)
                } else {
                    None
                }
            },
            TerrainType::Grass => {
                // Check neighbors for seeds (Trees)
                let has_seed_source = check_neighbors_for_tree(&grid, x, y);

                if has_seed_source && rng.gen_bool(seed_spread_chance) {
                    Some(TerrainType::Sapling)
                } else if rng.gen_bool(shrub_chance) {
                    // Occasional random shrub
                    Some(TerrainType::Shrub)
                } else {
                    None
                }
            },
            TerrainType::Sapling => {
                if rng.gen_bool(maturation_chance) {
                    Some(TerrainType::Tree)
                } else {
                    None
                }
            },
            _ => None,
        };

        if let Some(t) = new_type {
            grid.set(x, y, t);
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn check_neighbors_for_tree(grid: &TerrainGrid, x: usize, y: usize) -> bool {
    let x = x as i32;
    let y = y as i32;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && ny >= 0 {
                #[allow(clippy::cast_sign_loss)]
                if grid.get(nx as usize, ny as usize) == Some(TerrainType::Tree) {
                    return true;
                }
            }
        }
    }
    false
}
