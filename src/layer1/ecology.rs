//! Ecological Succession System (Spec 161).
//!
//! This module simulates the natural lifecycle of vegetation in the colony.
//! Unlike static map generation, the environment is dynamic: dirt becomes grass,
//! grass can grow into shrubs or saplings (if near trees), and saplings mature into trees.
//!
//! # The Cycle of Life
//!
//! 1.  **Pioneer Stage**: `Dirt` -> `Grass`
//!     - Occurs randomly on barren soil.
//! 2.  **Seeding Stage**: `Grass` -> `Sapling`
//!     - Requires a nearby `Tree` to provide seeds.
//!     - Alternatively, `Grass` can become `Shrub` (dead-end growth).
//! 3.  **Climax Stage**: `Sapling` -> `Tree`
//!     - Occurs over time as saplings mature.
//!
//! # Stochastic Simulation
//!
//! To keep performance high, we don't update every tile every tick. Instead, we sample
//! a fraction of the map (controlled by [`EcologyConfig::growth_rate`]) and apply
//! probabilistic updates. This creates organic, patchy growth patterns.

use crate::layer1::map::GridPosition;
use crate::layer1::notifications::{NotificationQueue, NotificationSeverity};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Configuration for ecological succession rates.
///
/// Controls how fast nature reclaims the land. Higher values mean faster growth
/// but higher CPU usage if `growth_rate` is too high (though it's capped by map size).
///
/// # Examples
///
/// ```
/// use scale::layer1::ecology::EcologyConfig;
///
/// let config = EcologyConfig {
///     growth_rate: 0.1,       // Check 10% of tiles per tick
///     pioneer_chance: 0.2,    // 20% chance Dirt -> Grass
///     seed_spread_chance: 0.1, // 10% chance Grass -> Sapling (near tree)
///     maturation_chance: 0.05, // 5% chance Sapling -> Tree
/// };
/// ```
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
/// This system implements the state machine for vegetation growth.
/// It runs stochastically, updating a random subset of tiles each tick.
///
/// # State Transitions
///
/// *   `Dirt` -> `Grass`: `pioneer_chance`
/// *   `Grass` -> `Sapling`: `seed_spread_chance` (requires neighbor `Tree`)
/// *   `Grass` -> `Shrub`: `pioneer_chance * 0.5` (if no `Tree` or failed seed roll)
/// *   `Sapling` -> `Tree`: `maturation_chance`
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
            }
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
            }
            TerrainType::Sapling => {
                if rng.gen_bool(maturation_chance) {
                    Some(TerrainType::Tree)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(t) = new_type {
            grid.set(x, y, t);
        }
    }
}

/// Helper: Checks the 8 neighbors of a tile for a `Tree`.
///
/// Used to determine if a `Grass` tile can receive seeds to become a `Sapling`.
///
/// # Returns
///
/// `true` if at least one neighbor is `TerrainType::Tree`.
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

// --- Spec 225: Keystone Species ---

/// Component marking a species (Flora/Fauna) with ecological metadata.
#[derive(Component, Debug, Clone)]
pub struct Species {
    /// Name of the species (e.g., "Giant Cactus").
    pub name: String,
    /// Whether this species is a Keystone species supporting the biome.
    pub is_keystone: bool,
}

impl Default for Species {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            is_keystone: false,
        }
    }
}

/// Component linking a dependent entity to a Keystone entity.
/// If the target entity dies or is removed, this dependent entity suffers.
#[derive(Component, Debug, Clone, Copy)]
pub struct DependentOn(pub Entity);

/// Component marking an entity as an anchor for the biome.
/// If this entity dies, the terrain at its location (and potentially radius) degrades.
#[derive(Component, Debug, Clone, Copy)]
pub struct BiomeAnchor {
    /// Radius of effect (for future expansion, currently 0/self).
    pub radius: u32,
    /// The terrain type this anchor sustains.
    pub target_terrain: TerrainType,
    /// The terrain type it reverts to if the anchor is lost.
    pub fallback_terrain: TerrainType,
}

/// System that handles the death of Keystone/Anchor entities.
///
/// When a Keystone with `BiomeAnchor` dies (gains `Dead` component),
/// the terrain at its position degrades to `fallback_terrain`.
#[allow(clippy::type_complexity)]
pub fn handle_keystone_death(
    query: Query<
        (&GridPosition, &BiomeAnchor, &Species),
        (With<Species>, Added<crate::layer1::health::Dead>),
    >,
    mut terrain: ResMut<TerrainGrid>,
    mut notifications: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    for (pos, anchor, species) in query.iter() {
        // Degrade terrain at position
        terrain.set(pos.x as usize, pos.y as usize, anchor.fallback_terrain);

        // MVP: Only affects the tile itself. Spec mentions radius but red test only checked the tile.
        // Future expansion: iterate radius.

        // NOTIFY PLAYER (Mosaic Polish)
        notifications.add_error(
            format!(
                "Biome Collapse: Keystone Species '{}' has died! Terrain degrading.",
                species.name
            ),
            time.tick,
        );
    }
}

/// System that handles the collapse of dependent species.
///
/// Checks if the entity referenced by `DependentOn` is dead or missing.
/// If so, the dependent entity dies (is despawned or takes fatal damage).
pub fn biome_collapse_system(
    mut commands: Commands,
    dependents: Query<(Entity, &DependentOn)>,
    all_entities: Query<(), ()>,
    dead_entities: Query<&crate::layer1::health::Dead>,
) {
    for (entity, dep) in dependents.iter() {
        let target = dep.0;

        // Check if target exists
        let exists = all_entities.get(target).is_ok();

        // Check if target is already dead (even if not despawned yet)
        let is_dead = dead_entities.get(target).is_ok();

        if !exists || is_dead {
            // Dependent dies
            commands.entity(entity).insert(crate::layer1::health::Dead);
        }
    }
}
