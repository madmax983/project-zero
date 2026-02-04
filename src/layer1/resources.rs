//! Resource management and mining mechanics.
//!
//! This module defines the colony's economic backbone: `ColonyResources` and the
//! mechanisms to extract them from the environment (e.g., `mine_rock`).
//!
//! # Key Concepts
//!
//! * **ColonyResources**: The global stockpile of Food, Wood, and Stone.
//! * **WorkProgress**: Tracks progress of multi-tick tasks like Mining or Forestry.
//! * **Mining/Forestry**: Functions to advance work on designations.

use crate::layer1::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Tracks the resources available to the colony.
///
/// This resource serves as the "bank" for the simulation.
#[derive(Resource, Default, Debug)]
pub struct ColonyResources {
    /// Total food available in the colony.
    pub food: f32,
    /// Total wood available in the colony.
    pub wood: f32,
    /// Total stone available in the colony.
    pub stone: f32,
}

/// Component tracking the progress of a work designation (mining, chopping, etc).
///
/// Attached to entities that are being actively worked on. The simulation uses this
/// to persist work across multiple ticks/frames.
#[derive(Component, Debug)]
pub struct WorkProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the task.
    pub max: f32,
}

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

/// Alias for backward compatibility.
pub type MiningProgress = WorkProgress;
/// Alias for backward compatibility.
pub type ForestryProgress = WorkProgress;

/// Applies work to a mining designation.
///
/// This function is the core of the mining mechanic. It advances the `WorkProgress`
/// of a specific designation. If the work completes the task, it:
/// 1. Despawns the designation.
/// 2. Changes the terrain from `Rock` to `Dirt`.
/// 3. Adds `1.0` Stone to `ColonyResources`.
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
        let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
        if idx < terrain.tiles.len() {
            terrain.tiles[idx] = TerrainType::Dirt;
        }

        // Add resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.stone += 1.0;

        // Remove designation
        world.despawn(designation_entity);
    }
}

/// Applies work to a forestry designation (chopping trees).
///
/// Simulates chopping a tree. On completion:
/// 1. Despawns the designation.
/// 2. Changes the terrain from `Tree` to `Dirt` (cleared land).
/// 3. Adds `1.0` Wood to `ColonyResources`.
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
        resources.wood += 1.0;

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

    #[test]
    fn test_forestry_uses_work_progress() {
        // Verify ForestryProgress is alias or we use WorkProgress
        let progress = WorkProgress {
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
                WorkProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Perform work
        chop_tree(&mut world, designation, 1.0);

        let progress = world.get::<WorkProgress>(designation).unwrap();
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
                WorkProgress {
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
