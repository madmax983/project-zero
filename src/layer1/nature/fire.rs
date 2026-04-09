#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::collapsible_if
)]

use crate::layer1::structure::Structure;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::GridPosition;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashSet;

/// Component representing a fire instance.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fire {
    /// How many ticks the fire will last before burning out.
    pub lifetime: u32,
    /// Intensity of the fire (currently unused, but good for future expansion).
    pub intensity: f32,
}

impl Default for Fire {
    fn default() -> Self {
        Self {
            lifetime: 50, // ticks
            intensity: 1.0,
        }
    }
}

/// Component making an entity susceptible to fire.
#[derive(Component, Debug, Clone, Copy)]
pub struct Flammable {
    /// Amount of fuel available for the fire.
    pub fuel: f32,
    /// Resistance to catching fire (0.0 = no resistance).
    pub fire_resistance: f32,
}

impl Default for Flammable {
    fn default() -> Self {
        Self {
            fuel: 100.0,
            fire_resistance: 0.0,
        }
    }
}

/// System that handles fire spreading to adjacent tiles.
pub fn fire_spread_system(world: &mut World) {
    let mut new_fires = Vec::new();
    let mut fire_locations = HashSet::new();

    // 1. Collect all current fire locations to avoid re-igniting or double-checking
    let mut fire_query = world.query::<(&GridPosition, &Fire)>();
    for (pos, _) in fire_query.iter(world) {
        fire_locations.insert(*pos);
    }

    // 2. Check neighbors for potential spread
    // We need to access terrain, so we scope the immutable borrow here.
    // ⚡ Bolt Optimization:
    // We avoid cloning the massive `terrain.tiles` Vec here. Instead, we query
    // the resource per neighbor check, eliminating an O(N) allocation per frame.
    let (width, height) = {
        let terrain = world.resource::<TerrainGrid>();
        (terrain.width, terrain.height)
    };

    let mut rng = rand::thread_rng();

    // ⚡ Bolt Optimization:
    // Iterating directly over `&fire_locations` avoids allocating a new `Vec` per frame, reducing heap allocations and memory churn.
    for &pos in &fire_locations {
        let neighbors = [
            (pos.x + 1, pos.y),
            (pos.x - 1, pos.y),
            (pos.x, pos.y + 1),
            (pos.x, pos.y - 1),
        ];

        for (nx, ny) in neighbors {
            let n_pos = GridPosition { x: nx, y: ny };

            // Skip if already burning
            if fire_locations.contains(&n_pos) {
                continue;
            }
            // Skip if out of bounds
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }

            let mut should_ignite = false;

            // Check Terrain
            // ⚡ Bolt Optimization: Immutable borrow of the `TerrainGrid` resource dynamically here avoids cloning.
            let is_tree = {
                let terrain = world.resource::<TerrainGrid>();
                let idx = (ny as usize) * width + (nx as usize);
                idx < terrain.tiles.len() && terrain.tiles[idx] == TerrainType::Tree
            };

            if is_tree {
                // Tree flammability chance
                if rng.gen_bool(0.1) {
                    // 10% chance per tick per neighbor
                    should_ignite = true;
                }
            }

            // Check Buildings (Flammable)
            if !should_ignite {
                let mut building_query = world.query::<(&GridPosition, &Flammable)>();
                for (b_pos, _) in building_query.iter(world) {
                    if *b_pos == n_pos {
                        if rng.gen_bool(0.1) {
                            should_ignite = true;
                        }
                        break;
                    }
                }
            }

            if should_ignite {
                // Check if we already queued a fire for this spot (avoid dupes in same frame)
                if !new_fires.contains(&n_pos) {
                    new_fires.push(n_pos);
                }
            }
        }
    }

    // 3. Spawn new fires
    for pos in new_fires {
        world.spawn((Fire::default(), pos));
    }
}

/// System that handles fire damage and burning out.
pub fn fire_damage_system(world: &mut World) {
    // Decrement lifetime, destroy burnt objects
    let mut fires_to_remove = Vec::new();
    let mut terrain_changes = Vec::new(); // (x, y, NewType)
    let mut burnt_entities = Vec::new(); // Entities at position to check for destruction

    let mut query = world.query::<(Entity, &GridPosition, &mut Fire)>();
    for (entity, pos, mut fire) in query.iter_mut(world) {
        if fire.lifetime > 0 {
            fire.lifetime -= 1;
        }

        if fire.lifetime == 0 {
            fires_to_remove.push((entity, *pos));
            burnt_entities.push(*pos);
        }
    }

    // Apply destruction for burnt-out fires
    let mut entities_to_despawn = Vec::new();
    {
        let terrain = world.resource::<TerrainGrid>();

        // Process removals and terrain/building destruction
        for (entity, pos) in &fires_to_remove {
            entities_to_despawn.push(*entity);

            // If it was on a tree, turn to dirt
            if let Some(tile) = terrain.get(pos.x as usize, pos.y as usize) {
                if tile == TerrainType::Tree {
                    terrain_changes.push((pos.x, pos.y, TerrainType::Dirt));
                }
            }
        }
    }

    for entity in entities_to_despawn {
        world.despawn(entity);
    }

    // Apply terrain changes
    if !terrain_changes.is_empty() {
        let mut terrain_mut = world.resource_mut::<TerrainGrid>();
        for (x, y, new_type) in terrain_changes {
            let idx = (y as usize) * terrain_mut.width + (x as usize);
            if idx < terrain_mut.tiles.len() {
                terrain_mut.tiles[idx] = new_type;
            }
        }
    }

    // Destroy flammable entities at burnt locations
    // We do this by finding all Flammable entities at the burnt positions
    // This requires a query scan which is O(N_buildings * N_burnt_tiles), ok for MVP
    if !burnt_entities.is_empty() {
        let mut entities_to_destroy = Vec::new();
        // Check for Structure component to avoid destroying durable buildings
        let mut query = world.query::<(Entity, &GridPosition, &Flammable, Option<&Structure>)>();

        for (entity, pos, _flammable, structure) in query.iter(world) {
            if burnt_entities.contains(pos) {
                // If it's a structure, it survives the fire burning out (unless HP was 0, handled elsewhere)
                if structure.is_some() {
                    continue;
                }
                entities_to_destroy.push(entity);
            }
        }

        for entity in entities_to_destroy {
            world.despawn(entity);
        }
    }
}

/// System to extinguish fire in low pressure (vacuum).
pub fn fire_pressure_check_system(world: &mut World) {
    use crate::layer1::pressure::PressureGrid;

    // ⚡ Bolt Optimization:
    // We avoid allocating an intermediate `Vec` (`fire_positions`) to store query results before processing.
    // We use `resource_scope` to allow querying the `World` while safely accessing `PressureGrid`.
    let mut to_despawn = Vec::new();

    let mut query = world.query_filtered::<(Entity, &GridPosition), With<Fire>>();

    // Try to get resource safely and if it doesn't exist, we just skip.
    if world.get_resource::<PressureGrid>().is_none() {
        return;
    }

    world.resource_scope(|world, pressure: Mut<PressureGrid>| {
        for (e, pos) in query.iter(world) {
            if pressure.get(pos.x, pos.y) < 0.1 {
                to_despawn.push(e);
            }
        }
    });

    // Despawn extinguished fires
    for e in to_despawn {
        world.despawn(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::GridPosition;

    #[test]
    fn test_flammable_component_defaults() {
        let f = Flammable::default();
        assert!(f.fuel > 0.0);
        assert!(f.fire_resistance >= 0.0);
    }

    #[test]
    fn test_fire_component_defaults() {
        let f = Fire::default();
        assert!(f.lifetime > 0);
        assert!(f.intensity > 0.0);
    }

    #[test]
    fn test_fire_spreads_to_tree() {
        let mut world = World::new();
        // 10x10 Grid
        let mut tiles = vec![TerrainType::Grass; 100];
        // (5,5) has Fire
        // (5,6) is Tree
        tiles[65] = TerrainType::Tree;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Spawn Fire at (5,5)
        world.spawn((
            Fire {
                intensity: 1.0,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system enough times to guarantee spread (since it's probabilistic, we might need to force it or mock RNG in impl, or just run many times)
        // For test stability, we usually mock RNG or set probability to 1.0 in a config resource.
        // Assuming we can control it or it happens eventually:
        for _ in 0..50 {
            // Increased iterations to be safe with RNG
            fire_spread_system(&mut world);
        }

        // Check if fire exists at (5,6)
        let has_fire = world
            .query::<(&GridPosition, &Fire)>()
            .iter(&world)
            .any(|(pos, _)| pos.x == 5 && pos.y == 6);

        assert!(has_fire, "Fire should spread to adjacent tree");
    }

    #[test]
    fn test_fire_spreads_to_flammable_building() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Fire at (5,5)
        world.spawn((
            Fire {
                intensity: 1.0,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Housing at (5,6) - Flammable
        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            Flammable::default(),
            GridPosition { x: 5, y: 6 },
        ));

        for _ in 0..200 {
            // Increased iterations to ensure RNG hits
            fire_spread_system(&mut world);
        }

        let has_fire = world
            .query::<(&GridPosition, &Fire)>()
            .iter(&world)
            .any(|(pos, _)| pos.x == 5 && pos.y == 6);

        assert!(
            has_fire,
            "Fire should spread to adjacent flammable building"
        );
    }

    #[test]
    fn test_fire_does_not_spread_to_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[65] = TerrainType::Rock; // (5,6)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        world.spawn((
            Fire {
                intensity: 1.0,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        for _ in 0..10 {
            fire_spread_system(&mut world);
        }

        let has_fire = world
            .query::<(&GridPosition, &Fire)>()
            .iter(&world)
            .any(|(pos, _)| pos.x == 5 && pos.y == 6);

        assert!(!has_fire, "Fire should not spread to rock");
    }

    #[test]
    fn test_fire_burns_out_and_destroys_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5,5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Fire at (5,5) with short lifetime
        let fire_entity = world
            .spawn((
                Fire {
                    lifetime: 1,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run damage system (decrements lifetime)
        // Lifetime is 1, so one tick should set it to 0
        fire_damage_system(&mut world);

        // Fire should be gone (lifetime 0 -> remove)
        // Wait, logic says if fire.lifetime == 0 { remove }.
        // If start with 1, 1 -> 0, then remove.
        assert!(world.get_entity(fire_entity).is_err());

        // Terrain should be Dirt (burnt tree)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));
    }

    #[test]
    fn test_fire_extinguished_in_vacuum() {
        use crate::layer1::pressure::PressureGrid;
        use bevy_ecs::system::RunSystemOnce;
        let mut world = World::new();

        let mut pressure = PressureGrid::new(10, 10);
        pressure.set(5, 5, 0.0); // Vacuum
        world.insert_resource(pressure);

        let fire_entity = world
            .spawn((Fire::default(), GridPosition { x: 5, y: 5 }))
            .id();

        world
            .run_system_once(fire_pressure_check_system)
            .expect("System should run successfully");

        assert!(
            world.get_entity(fire_entity).is_err(),
            "Fire should be extinguished in vacuum"
        );
    }

    #[test]
    fn test_fire_survives_in_pressure() {
        use crate::layer1::pressure::PressureGrid;
        use bevy_ecs::system::RunSystemOnce;
        let mut world = World::new();

        let mut pressure = PressureGrid::new(10, 10);
        pressure.set(5, 5, 1.0); // Normal pressure
        world.insert_resource(pressure);

        let fire_entity = world
            .spawn((Fire::default(), GridPosition { x: 5, y: 5 }))
            .id();

        world
            .run_system_once(fire_pressure_check_system)
            .expect("System should run successfully");

        assert!(
            world.get_entity(fire_entity).is_ok(),
            "Fire should survive in normal pressure"
        );
    }
}
