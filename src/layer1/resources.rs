use crate::layer1::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Tracks the resources available to the colony.
#[derive(Resource, Default, Debug)]
pub struct ColonyResources {
    /// Total food available in the colony.
    pub food: f32,
    /// Total wood available in the colony.
    pub wood: f32,
    /// Total stone available in the colony.
    pub stone: f32,
}

/// Component tracking the progress of a mining designation.
#[derive(Component, Debug)]
pub struct MiningProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the mining.
    pub max: f32,
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

/// Applies work to a mining designation.
/// If complete, transforms terrain and awards resources.
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
        resources.stone += 1.0;

        // Remove designation
        world.despawn(designation_entity);
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_colony_resources_fields() {
        let resources = ColonyResources::default();
        // Check for new fields
        assert_eq!(resources.food, 0.0);
        assert_eq!(resources.wood, 0.0);
        assert_eq!(resources.stone, 0.0);
    }

    #[test]
    fn test_mining_progress_component() {
        let progress = MiningProgress {
            current: 0.0,
            max: 100.0,
        };
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.max, 100.0);
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
        assert_eq!(progress.current, 1.0);
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
        assert_eq!(resources.stone, 1.0);
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
        assert_eq!(progress.current, 0.0);
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
        assert_eq!(progress.current, 0.0);
    }
}
