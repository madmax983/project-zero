//! Resource management and mining mechanics.
//!
//! This module defines the colony's economic backbone: `ColonyResources` and the
//! mechanisms to extract them from the environment (e.g., `mine_rock`).
//!
//! # Key Concepts
//!
//! * **ColonyResources**: The global stockpile of Food, Wood, and Stone.
//! * **Mining**: A multi-tick process tracked by `MiningProgress` that converts
//!   terrain (Rock -> Dirt) and yields resources (Stone).
//!
//! # The Mining Loop
//!
//! 1. Player designates a tile (see `crate::layer1::designation`).
//! 2. A pop is assigned the job (see `crate::layer1::pop`).
//! 3. The pop works on the tile, calling `mine_rock`.
//! 4. `MiningProgress` accumulates.
//! 5. Upon completion, the tile changes and resources are awarded.

use crate::layer1::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Tracks the resources available to the colony.
///
/// This resource serves as the "bank" for the simulation.
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::ColonyResources;
///
/// let mut resources = ColonyResources::default();
/// resources.food += 10.0;
/// assert_eq!(resources.food, 10.0);
/// ```
#[derive(Resource, Debug)]
pub struct ColonyResources {
    /// Total food available in the colony.
    pub food: f32,
    /// Total wood available in the colony.
    pub wood: f32,
    /// Total stone available in the colony.
    pub stone: f32,
    /// Maximum food capacity.
    pub max_food: f32,
    /// Maximum wood capacity.
    pub max_wood: f32,
    /// Maximum stone capacity.
    pub max_stone: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            food: 0.0,
            wood: 0.0,
            stone: 0.0,
            max_food: 50.0,
            max_wood: 50.0,
            max_stone: 20.0,
        }
    }
}

impl ColonyResources {
    /// Adds wood, clamping to the maximum capacity.
    pub fn add_wood(&mut self, amount: f32) {
        self.wood = (self.wood + amount).min(self.max_wood);
    }

    /// Adds stone, clamping to the maximum capacity.
    pub fn add_stone(&mut self, amount: f32) {
        self.stone = (self.stone + amount).min(self.max_stone);
    }

    /// Adds food, clamping to the maximum capacity.
    pub fn add_food(&mut self, amount: f32) {
        self.food = (self.food + amount).min(self.max_food);
    }

    /// Checks if the colony can afford the given cost.
    ///
    /// # Parameters
    ///
    /// * `cost`: The resources required.
    ///
    /// # Returns
    ///
    /// True if all resources are sufficient.
    #[must_use]
    pub fn can_afford(&self, cost: &Self) -> bool {
        self.food >= cost.food && self.wood >= cost.wood && self.stone >= cost.stone
    }

    /// Deducts the given cost from the colony's resources.
    ///
    /// # Parameters
    ///
    /// * `cost`: The resources to deduct.
    pub fn deduct(&mut self, cost: &Self) {
        self.food -= cost.food;
        self.wood -= cost.wood;
        self.stone -= cost.stone;
    }
}

/// Component tracking the progress of a mining designation.
///
/// Attached to entities that are being actively mined. The simulation uses this
/// to persist work across multiple ticks/frames.
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::MiningProgress;
///
/// let progress = MiningProgress { current: 50.0, max: 100.0 };
/// assert!(!progress.is_complete());
/// ```
#[derive(Component, Debug)]
pub struct MiningProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the mining.
    pub max: f32,
}

/// Base maximum food capacity.
pub const BASE_MAX_FOOD: f32 = 50.0;
/// Base maximum wood capacity.
pub const BASE_MAX_WOOD: f32 = 50.0;
/// Base maximum stone capacity.
pub const BASE_MAX_STONE: f32 = 20.0;

impl MiningProgress {
    /// Returns true if the work is finished.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

/// Component tracking the progress of a forestry designation.
#[derive(Component, Debug, Default)]
pub struct ForestryProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the chopping.
    pub max: f32,
}

impl ForestryProgress {
    /// Returns true if the work is finished.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

/// Applies work to a mining designation.
///
/// This function is the core of the mining mechanic. It advances the `MiningProgress`
/// of a specific designation. If the work completes the task, it:
/// 1. Despawns the designation.
/// 2. Changes the terrain from `Rock` to `Dirt`.
/// 3. Adds `1.0` Stone to `ColonyResources`.
///
/// # Parameters
///
/// * `world`: Mutable access to the ECS world (needed to modify terrain and resources).
/// * `designation_entity`: The entity ID of the designation being worked on.
/// * `work_amount`: How much progress to add (usually based on worker skill/speed).
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::{mine_rock, ColonyResources, MiningProgress};
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use scale::layer1::GridPosition;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
///
/// // 1. Setup World
/// let mut tiles = vec![TerrainType::Grass; 100];
/// tiles[0] = TerrainType::Rock; // Target is rock
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(ColonyResources::default());
///
/// // 2. Create Designation
/// let designation = world.spawn((
///     GridPosition { x: 0, y: 0 },
///     MiningProgress { current: 0.0, max: 10.0 }
/// )).id();
///
/// // 3. Work until done
/// mine_rock(&mut world, designation, 10.0);
///
/// // 4. Verify Result
/// let resources = world.resource::<ColonyResources>();
/// assert_eq!(resources.stone, 1.0);
/// ```
#[allow(clippy::cast_sign_loss)]
pub fn mine_rock(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify terrain
    let (pos, is_rock) = {
        let pos = if let Some(p) = world.get::<GridPosition>(designation_entity) {
            *p
        } else {
            return;
        };

        if pos.x < 0 || pos.y < 0 {
            return;
        }

        let terrain = world.resource::<TerrainGrid>();
        // Safe to cast because we checked for negative above
        let is_rock = terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Rock);
        (pos, is_rock)
    };

    if !is_rock {
        return;
    }

    // 2. Update progress
    let completed = if let Some(mut progress) = world.get_mut::<MiningProgress>(designation_entity)
    {
        progress.current += work_amount;
        progress.current >= progress.max
    } else {
        false
    };

    // 3. Handle completion
    if completed {
        // Change terrain
        let mut terrain = world.resource_mut::<TerrainGrid>();
        // Check bounds again? Technically redundant if terrain didn't shrink, but safe.
        // Also we checked < 0 earlier.
        let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
        if idx < terrain.tiles.len() {
            terrain.tiles[idx] = TerrainType::Dirt;
        }

        // Add resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add_stone(1.0);

        // Remove designation
        world.despawn(designation_entity);
    }
}

/// Applies work to a forestry designation (chopping a tree).
///
/// Similar to `mine_rock`, but for trees.
///
/// # Parameters
///
/// * `world`: Mutable access to the ECS world.
/// * `designation_entity`: The entity ID of the designation.
/// * `work_amount`: How much progress to add.
#[allow(clippy::cast_sign_loss)]
pub fn chop_tree(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify terrain
    let (pos, is_tree) = {
        let pos = if let Some(p) = world.get::<GridPosition>(designation_entity) {
            *p
        } else {
            return;
        };

        if pos.x < 0 || pos.y < 0 {
            return;
        }

        let terrain = world.resource::<TerrainGrid>();
        let is_tree = terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Tree);
        (pos, is_tree)
    };

    if !is_tree {
        return;
    }

    // 2. Update progress
    let completed =
        if let Some(mut progress) = world.get_mut::<ForestryProgress>(designation_entity) {
            progress.current += work_amount;
            progress.current >= progress.max
        } else {
            false
        };

    // 3. Handle completion
    if completed {
        // Change terrain
        let mut terrain = world.resource_mut::<TerrainGrid>();
        let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
        if idx < terrain.tiles.len() {
            terrain.tiles[idx] = TerrainType::Dirt;
        }

        // Add resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add_wood(1.0);

        // Remove designation
        world.despawn(designation_entity);
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_colony_resources_fields() {
        let resources = ColonyResources::default();
        // Check for new fields
        assert!((resources.food - 0.0).abs() < f32::EPSILON);
        assert!((resources.wood - 0.0).abs() < f32::EPSILON);
        assert!((resources.stone - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mining_progress_component() {
        let progress = MiningProgress {
            current: 0.0,
            max: 100.0,
        };
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
        assert!((progress.max - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_increments_progress() {
        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Setup Resources
        world.insert_resource(ColonyResources::default());

        // Spawn Designation with MiningProgress
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Perform work (simulate 1 tick of work)
        mine_rock(&mut world, designation, 1.0);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!((progress.current - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_completion() {
        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 9.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Complete the work
        mine_rock(&mut world, designation, 1.0);

        // 1. Entity should be despawned (Designation removed)
        assert!(world.get_entity(designation).is_err());

        // 2. Terrain should be Dirt
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert!((resources.stone - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_ignores_non_rock() {
        let mut world = World::new();
        // Setup Grass tile (cannot mine grass for stone)
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        mine_rock(&mut world, designation, 5.0);

        // Should not progress
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_no_position() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                // No GridPosition
            ))
            .id();

        mine_rock(&mut world, designation, 1.0);

        // Should just return, no panic
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mining_progress_is_complete() {
        let p = MiningProgress {
            current: 10.0,
            max: 10.0,
        };
        assert!(p.is_complete());

        let p2 = MiningProgress {
            current: 5.0,
            max: 10.0,
        };
        assert!(!p2.is_complete());
    }

    #[test]
    fn test_forestry_progress_component() {
        let progress = ForestryProgress {
            current: 0.0,
            max: 50.0,
        };
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.max, 50.0);
    }

    #[test]
    fn test_chop_tree_increments_progress() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                ForestryProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Perform work
        chop_tree(&mut world, designation, 1.0);

        let progress = world.get::<ForestryProgress>(designation).unwrap();
        assert_eq!(progress.current, 1.0);
    }

    #[test]
    fn test_chop_tree_completion() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                ForestryProgress {
                    current: 9.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Complete work
        chop_tree(&mut world, designation, 1.0);

        // 1. Entity should be despawned
        assert!(world.get_entity(designation).is_err());

        // 2. Terrain should be Dirt (cleared land)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. Resources should increase (Wood)
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 1.0);
    }
}
