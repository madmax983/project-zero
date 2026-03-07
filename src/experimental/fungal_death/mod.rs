//! Fungal Decomposition (Nova Feature).
//!
//! # The Spark
//! We have a `Corpse` decay system, and a `TerrainGrid` that supports `Sapling` and `Shrub`.
//! What if death brought new life directly to the environment?
//!
//! # The Feature
//! The `fungal_death_system` watches for corpses that are almost fully decayed (decay > 0.9).
//! When found, it turns the terrain beneath them into a `Sapling` (or `Shrub`), symbolizing
//! fungal and botanical life taking root in the biomass.

use crate::layer1::funeral::Corpse;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// System that transforms the terrain beneath heavily decayed corpses into plant life.
pub fn fungal_death_system(
    mut terrain: ResMut<TerrainGrid>,
    corpses: Query<(&GridPosition, &Corpse)>,
) {
    for (pos, corpse) in corpses.iter() {
        if corpse.decay > 0.9 {
            // Check if terrain is valid for planting (e.g. Dirt, Grass)
            if let Some(tile) = terrain.get(pos.x as usize, pos.y as usize) {
                if tile == TerrainType::Dirt || tile == TerrainType::Grass {
                    // Turn to Sapling
                    terrain.set(pos.x as usize, pos.y as usize, TerrainType::Sapling);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(fungal_death_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_corpse_spawns_sapling() {
        let mut world = World::new();

        // Setup a 2x2 dirt terrain
        let mut terrain = crate::layer1::terrain::generate_terrain(2, 2);
        terrain.set(0, 0, TerrainType::Dirt);
        terrain.set(1, 0, TerrainType::Rock);
        world.insert_resource(terrain);

        // Add a highly decayed corpse on Dirt
        world.spawn((
            GridPosition { x: 0, y: 0 },
            Corpse {
                name: "Bob".to_string(),
                decay: 0.95,
            },
        ));

        // Add a slightly decayed corpse on Dirt (should do nothing)
        world.spawn((
            GridPosition { x: 0, y: 1 }, // Note: (0, 1) is Grass by default if not set, let's assume valid
            Corpse {
                name: "Alice".to_string(),
                decay: 0.5,
            },
        ));

        // Add a highly decayed corpse on Rock (should do nothing)
        world.spawn((
            GridPosition { x: 1, y: 0 },
            Corpse {
                name: "Charlie".to_string(),
                decay: 0.95,
            },
        ));

        world.run_system_once(fungal_death_system).unwrap();

        let t = world.resource::<TerrainGrid>();

        // Highly decayed on Dirt -> Sapling
        assert_eq!(t.get(0, 0).unwrap(), TerrainType::Sapling);

        // Slightly decayed -> No change (Grass default)
        assert_ne!(t.get(0, 1).unwrap(), TerrainType::Sapling);

        // Highly decayed on Rock -> No change (Rock)
        assert_eq!(t.get(1, 0).unwrap(), TerrainType::Rock);
    }
}
