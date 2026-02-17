//! The Blob: An expanding hazard.
//!
//! Blobs are semi-indestructible entities that consume resources and damage buildings.
//! They spread to adjacent tiles over time.

use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ResourceItem;
use crate::layer1::structure::Structure;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};

/// A slow-moving, semi-indestructible hazard that consumes everything in its path.
#[derive(Component, Default, Debug, Clone)]
pub struct Blob {
    /// Ticks until next spread attempt.
    pub spread_timer: u32,
}

/// System that handles the spread of the Blob.
///
/// Blobs attempt to spread to adjacent tiles periodically.
/// If a building blocks the path, the Blob damages it.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn blob_spread_system(
    mut commands: Commands,
    mut blobs: Query<(&mut Blob, &GridPosition)>,
    mut buildings: Query<(Entity, &GridPosition, &mut Structure), With<Building>>,
    terrain: Res<TerrainGrid>,
) {
    let mut rng = rand::thread_rng();

    // 1. Build map of existing blobs to prevent stacking
    let mut blob_positions = HashSet::new();
    for (_, pos) in blobs.iter() {
        blob_positions.insert(*pos);
    }

    // 2. Build map of buildings for O(1) lookup
    let mut building_map = HashMap::new();
    for (entity, pos, _) in buildings.iter() {
        building_map.insert(*pos, entity);
    }

    for (mut blob, pos) in &mut blobs {
        if blob.spread_timer > 0 {
            blob.spread_timer -= 1;
            continue;
        }
        // Spread!
        blob.spread_timer = 10; // Reset timer

        // Pick random neighbor
        let (dx, dy) = match rng.gen_range(0..4) {
            0 => (0, 1),
            1 => (0, -1),
            2 => (1, 0),
            _ => (-1, 0),
        };
        let target_pos = GridPosition {
            x: pos.x + dx,
            y: pos.y + dy,
        };

        // Check bounds
        if target_pos.x < 0
            || target_pos.y < 0
            || target_pos.x >= terrain.width as i32
            || target_pos.y >= terrain.height as i32
        {
            continue;
        }

        // Check for building
        if let Some(&entity) = building_map.get(&target_pos) {
            if let Ok((_, _, mut structure)) = buildings.get_mut(entity) {
                structure.current_hp -= 1.0;
            }
            continue; // Attacked building, don't move/spawn
        }

        // Check for existing blob
        if blob_positions.contains(&target_pos) {
            continue; // Already occupied by blob
        }

        // Spawn new blob
        commands.spawn((
            Blob {
                spread_timer: 10 + rng.gen_range(0..5),
            },
            target_pos,
        ));

        blob_positions.insert(target_pos);
    }
}

/// System that handles consumption of items by the Blob.
///
/// If a Blob shares a tile with a resource item, the item is destroyed.
pub fn blob_consumption_system(
    mut commands: Commands,
    blobs: Query<&GridPosition, With<Blob>>,
    items: Query<(Entity, &GridPosition), With<ResourceItem>>,
) {
    // Optimize: Build a HashSet of blob positions
    let mut blob_positions = HashSet::new();
    for pos in blobs.iter() {
        blob_positions.insert(*pos);
    }

    for (entity, pos) in items.iter() {
        if blob_positions.contains(pos) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::blob::{Blob, blob_consumption_system, blob_spread_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{ResourceItem, ResourceType};
    use crate::layer1::structure::Structure;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_blob_spreads_to_empty_tile() {
        let mut world = World::new();
        // Setup: 1 Blob at (5,5)
        world.spawn((Blob { spread_timer: 0 }, GridPosition { x: 5, y: 5 }));

        // Mock resources (Map bounds etc) if needed
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Run spread system
        // We force timer to trigger
        let _ = world.run_system_once(blob_spread_system);

        // Should have more than 1 blob now
        let blob_count = world.query::<&Blob>().iter(&world).count();
        assert!(blob_count > 1, "Blob should spread");

        // Check adjacency (simplistic check)
        let positions: Vec<GridPosition> = world
            .query_filtered::<&GridPosition, With<Blob>>()
            .iter(&world)
            .cloned()
            .collect();
        assert!(positions.contains(&GridPosition { x: 5, y: 5 }));
        // New blob should be adjacent
        let new_pos = positions
            .iter()
            .find(|p| **p != GridPosition { x: 5, y: 5 })
            .unwrap();
        assert!((new_pos.x - 5).abs() <= 1 && (new_pos.y - 5).abs() <= 1);
    }

    #[test]
    fn test_blob_eats_items() {
        let mut world = World::new();
        // Blob and Item at same location
        world.spawn((Blob::default(), GridPosition { x: 5, y: 5 }));
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 1.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _ = world.run_system_once(blob_consumption_system);

        // Item should be despawned
        assert!(world.get_entity(item).is_err());
    }

    #[test]
    fn test_blob_damages_buildings() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        world.spawn((Blob { spread_timer: 0 }, GridPosition { x: 5, y: 5 }));

        // Walls everywhere around
        let walls = vec![(5, 6), (5, 4), (6, 5), (4, 5)];

        let mut wall_entities = Vec::new();
        for (x, y) in walls {
            let w = world
                .spawn((
                    Building {
                        building_type: BuildingType::Wall,
                    },
                    GridPosition { x, y },
                    Structure {
                        current_hp: 100.0,
                        max_hp: 100.0,
                    },
                ))
                .id();
            wall_entities.push(w);
        }

        let _ = world.run_system_once(blob_spread_system);

        // At least one wall should be damaged
        let damaged = wall_entities.iter().any(|&e| {
            let s = world.get::<Structure>(e).unwrap();
            s.current_hp < 100.0
        });

        assert!(damaged, "Blob should damage at least one adjacent building");
    }

    #[test]
    fn test_blob_blocked_by_walls() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        // Blob at (0,0)
        world.spawn((Blob { spread_timer: 0 }, GridPosition { x: 0, y: 0 }));

        // Wall at (0,1) and (1,0) (blocking spread)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 0, y: 1 },
            Structure {
                current_hp: 10.0,
                max_hp: 10.0,
            },
        ));
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 1, y: 0 },
            Structure {
                current_hp: 10.0,
                max_hp: 10.0,
            },
        ));

        // Mock OccupiedTiles logic
        let mut occupied = crate::layer1::building::OccupiedTiles::default();
        occupied.0.insert((0, 1));
        occupied.0.insert((1, 0));
        world.insert_resource(occupied);

        // Run multiple times to ensure it tries to spread
        for _ in 0..10 {
            let _ = world.run_system_once(blob_spread_system);
            // Reset timer manually to force attempt
            if let Some(mut blob) = world.query::<&mut Blob>().iter_mut(&mut world).next() {
                blob.spread_timer = 0;
            }
        }

        // Blob count should NOT increase (blocked by wall)
        let blob_count = world.query::<&Blob>().iter(&world).count();
        assert_eq!(blob_count, 1);
    }
}
