//! Dreams of Genesis (Somnambulistic Terraforming) - Nova Feature
//!
//! # The Spark
//! We have a `Sleepwalking` mental break that causes Pops to wander aimlessly.
//! What if, while sleepwalking, they subconsciously reshape the world to match their dreams?
//!
//! # The Feature
//! Pops that are `Sleepwalking` leave a trail of beauty. If they walk on Dirt or Rock,
//! there is a chance it turns into Grass. If they walk on Grass, there is a chance
//! they spontaneously construct a `FlowerBed`.
//!
//! # Potential
//! Turns a negative mental break into a chaotic, slightly beneficial side-effect that
//! physically scars the planet with the colony's dreams.

use crate::layer1::building::{
    spawn_building_with_material, BuildingType, MaterialType, OccupiedTiles,
};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::utility_ai::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::Rng;

const TERRAFORM_CHANCE: f64 = 0.05; // 5% chance per tick while sleepwalking

pub fn dreams_of_genesis_system(world: &mut World) {
    // Collect data to avoid borrow checker issues
    let mut sleepwalkers = Vec::new();

    // Scoped borrow of world to find sleepwalkers
    {
        let mut query = world.query_filtered::<(&PopAction, &GridPosition), With<Pop>>();
        for (action, pos) in query.iter(world) {
            if action.current == ActionType::Sleepwalking {
                sleepwalkers.push(*pos);
            }
        }
    }

    if sleepwalkers.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    for pos in sleepwalkers {
        if !rng.gen_bool(TERRAFORM_CHANCE) {
            continue;
        }

        // Check if we can terraform the tile
        let mut should_spawn_flower = false;

        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            if pos.x >= 0
                && (pos.x as usize) < terrain.width
                && pos.y >= 0
                && (pos.y as usize) < terrain.height
            {
                let idx = (pos.y as usize).checked_mul(terrain.width).and_then(|i| i.checked_add(pos.x as usize)).unwrap_or(usize::MAX);
                if idx < terrain.tiles.len() {
                    let tile = &mut terrain.tiles[idx];
                    if *tile == TerrainType::Dirt || *tile == TerrainType::Rock {
                        // Terraform to grass
                        *tile = TerrainType::Grass;
                    } else if *tile == TerrainType::Grass {
                        // Might spawn a flower bed
                        should_spawn_flower = true;
                    }
                }
            }
        }

        if should_spawn_flower {
            // Check if occupied
            let is_occupied = world
                .resource::<OccupiedTiles>()
                .0
                .contains(&(pos.x, pos.y));
            if !is_occupied {
                spawn_building_with_material(
                    world,
                    pos.x,
                    pos.y,
                    BuildingType::FlowerBed,
                    MaterialType::Wood, // Flower beds use wood/default material
                );
                world
                    .resource_mut::<OccupiedTiles>()
                    .0
                    .insert((pos.x, pos.y));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Dirt; 100],
        };
        world.insert_resource(terrain);
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::prototyping::BuildingMastery::default());
        world
    }

    #[test]
    fn test_sleepwalking_terraforms_dirt_to_grass() {
        let mut world = setup_world();
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Sleepwalking,
                ..Default::default()
            },
        ));

        // Run multiple times to overcome RNG
        let mut changed = false;
        for _ in 0..500 {
            world.run_system_once(dreams_of_genesis_system).unwrap();
            let terrain = world.resource::<TerrainGrid>();
            if terrain.get(5, 5) == Some(TerrainType::Grass) {
                changed = true;
                break;
            }
        }

        assert!(changed, "Dirt should be terraformed to Grass");
    }

    #[test]
    fn test_sleepwalking_spawns_flower_bed_on_grass() {
        let mut world = setup_world();

        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Grass; // Set (5, 5) to Grass
        }

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Sleepwalking,
                ..Default::default()
            },
        ));

        // Run multiple times to overcome RNG
        let mut spawned = false;
        for _ in 0..500 {
            world.run_system_once(dreams_of_genesis_system).unwrap();
            let occupied = world.resource::<OccupiedTiles>();
            if occupied.0.contains(&(5, 5)) {
                spawned = true;
                break;
            }
        }

        assert!(spawned, "Flower bed should be spawned on Grass");
    }

    #[test]
    fn test_normal_action_does_not_terraform() {
        let mut world = setup_world();
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
        ));

        for _ in 0..100 {
            world.run_system_once(dreams_of_genesis_system).unwrap();
        }

        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(
            terrain.get(5, 5),
            Some(TerrainType::Dirt),
            "Normal action should not terraform"
        );
    }
}
