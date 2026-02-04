//! Resource management and mining mechanics.
//!
//! This module defines the colony's economic backbone: `ColonyResources` and the
//! mechanisms to extract them from the environment (e.g., `mine_rock`).
//!
//! # Key Concepts
//!
//! * **ColonyResources**: The global stockpile of Food, Wood, and Stone.
//! * **Mining**: A multi-tick process tracked by `WorkProgress` that converts
//!   terrain (Rock -> Dirt) and yields resources (Stone).
//!
//! # The Mining Loop
//!
//! 1. Player designates a tile (see `crate::layer1::designation`).
//! 2. A pop is assigned the job (see `crate::layer1::pop`).
//! 3. The pop works on the tile, calling `mine_rock`.
//! 4. `WorkProgress` accumulates.
//! 5. Upon completion, the tile changes and resources are awarded.

use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::pop::Pop;
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
}

/// Component tracking the progress of a generic work designation (mining, chopping, etc.).
///
/// Attached to entities that are being actively worked on. The simulation uses this
/// to persist work across multiple ticks/frames.
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::WorkProgress;
///
/// let progress = WorkProgress { current: 50.0, max: 100.0 };
/// assert!(!progress.is_complete());
/// ```
#[derive(Component, Debug)]
pub struct WorkProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the task.
    pub max: f32,
}

/// Base maximum food capacity.
pub const BASE_MAX_FOOD: f32 = 50.0;
/// Base maximum wood capacity.
pub const BASE_MAX_WOOD: f32 = 50.0;
/// Base maximum stone capacity.
pub const BASE_MAX_STONE: f32 = 20.0;

impl WorkProgress {
    /// Returns true if the work is finished.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

impl Default for WorkProgress {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

/// Applies work to a forestry designation.
///
/// If work completes:
/// 1. Despawns designation.
/// 2. Changes Terrain Tree -> Dirt.
/// 3. Adds 1.0 Wood.
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
    let completed = if let Some(mut progress) = world.get_mut::<WorkProgress>(designation_entity) {
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

/// Applies work to a mining designation.
///
/// This function is the core of the mining mechanic. It advances the `WorkProgress`
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
/// use scale::layer1::resources::{mine_rock, ColonyResources, WorkProgress};
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
///     WorkProgress { current: 0.0, max: 10.0 }
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
    let completed = if let Some(mut progress) = world.get_mut::<WorkProgress>(designation_entity) {
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

/// System to process active work designations (Mining, Chopping).
///
/// Iterates over all designations with `WorkProgress`. If a `Pop` is within range,
/// work is performed.
pub fn process_work_system(world: &mut World) {
    let mut work_requests = Vec::new();

    // 1. Collect potential work targets
    // We can't mutate world while querying, so we collect needed data.
    // Query: (Entity, &Designation, &GridPosition, &WorkProgress)
    // Note: We only care about designations that HAVE WorkProgress.
    let mut designation_query = world.query::<(Entity, &Designation, &GridPosition, &WorkProgress)>();
    let mut pop_query = world.query::<(&Pop, &GridPosition)>();

    // We also need pops positions
    let pop_positions: Vec<GridPosition> = pop_query.iter(world).map(|(_, pos)| *pos).collect();

    for (entity, designation, pos, _) in designation_query.iter(world) {
        // Check if any pop is close enough
        // Range = 10 (Chebyshev or Manhattan? Spec 018 says "within MINING_RANGE (10)". Let's use Chebyshev/Max axis distance).
        let in_range = pop_positions.iter().any(|pop_pos| {
            (pop_pos.x - pos.x).abs().max((pop_pos.y - pos.y).abs()) <= 10
        });

        if in_range {
            work_requests.push((entity, designation.designation_type));
        }
    }

    // 2. Perform work
    const WORK_AMOUNT: f32 = 1.0;
    for (entity, work_type) in work_requests {
        match work_type {
            DesignationType::Mine => mine_rock(world, entity, WORK_AMOUNT),
            DesignationType::Chop => chop_tree(world, entity, WORK_AMOUNT),
            _ => {}
        }
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
    fn test_work_progress_component() {
        let progress = WorkProgress {
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

        // Spawn Designation with WorkProgress
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                WorkProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Perform work (simulate 1 tick of work)
        mine_rock(&mut world, designation, 1.0);

        let progress = world.get::<WorkProgress>(designation).unwrap();
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
                WorkProgress {
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
                WorkProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        mine_rock(&mut world, designation, 5.0);

        // Should not progress
        let progress = world.get::<WorkProgress>(designation).unwrap();
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
                WorkProgress {
                    current: 0.0,
                    max: 10.0,
                },
                // No GridPosition
            ))
            .id();

        mine_rock(&mut world, designation, 1.0);

        // Should just return, no panic
        let progress = world.get::<WorkProgress>(designation).unwrap();
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_work_progress_is_complete() {
        let p = WorkProgress {
            current: 10.0,
            max: 10.0,
        };
        assert!(p.is_complete());

        let p2 = WorkProgress {
            current: 5.0,
            max: 10.0,
        };
        assert!(!p2.is_complete());
    }

    // Forestry Tests from Spec 019
    use crate::layer1::designation::can_designate;
    use ratatui::style::Color;

    #[test]
    fn test_terrain_type_tree() {
        // Test new variant properties
        assert_eq!(TerrainType::Tree.as_str(), "↑");
        // Ratatui doesn't have DarkGreen, using RGB
        assert_eq!(TerrainType::Tree.color(), Color::Rgb(0, 100, 0));
        assert_eq!(TerrainType::Tree.name(), "Tree");
    }

    #[test]
    fn test_designation_type_chop() {
        // Test new variant properties
        assert_eq!(DesignationType::Chop.char(), '🪓'); // Axe character
        assert_eq!(DesignationType::Chop.label(), "Chop");
    }

    #[test]
    fn test_can_designate_chop_valid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Should be able to chop a Tree
        assert!(can_designate(&world, 5, 5, DesignationType::Chop));
    }

    #[test]
    fn test_can_designate_chop_invalid() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Cannot chop Grass
        assert!(!can_designate(&world, 5, 5, DesignationType::Chop));
    }

    #[test]
    fn test_chop_tree_increments_progress() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            WorkProgress { current: 0.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Perform work
        chop_tree(&mut world, designation, 1.0);

        let progress = world.get::<WorkProgress>(designation).unwrap();
        assert!((progress.current - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_chop_tree_completion() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            WorkProgress { current: 9.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Complete work
        chop_tree(&mut world, designation, 1.0);

        // 1. Entity should be despawned
        assert!(world.get_entity(designation).is_err());

        // 2. Terrain should be Dirt (cleared land)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. Resources should increase (Wood)
        let resources = world.resource::<ColonyResources>();
        assert!((resources.wood - 1.0).abs() < f32::EPSILON);
    }
}
