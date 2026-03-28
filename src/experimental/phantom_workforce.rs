//! The Phantom Workforce (Nova Feature).
//!
//! # The Spark
//! We have `Dead` logic, `Building` workplaces, and a `StressTracker` system.
//! What if Pops who die at work occasionally manifest as "Phantoms" that continue
//! to silently operate the machinery, producing resources but terrifying the living?
//!
//! # The Feature
//! When a Pop dies while actively assigned to a production building, there is a small
//! chance they leave behind a `PhantomWorker` entity bound to that building.
//! These phantoms passively generate resources, acting as highly efficient (but terrifying) free labor.
//! Living pops that get too close suffer massive stress penalties.

use crate::layer1::actions::AssignedTo;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::health::Dead;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component indicating an entity is a Phantom Worker bound to a specific building.
#[derive(Component)]
pub struct PhantomWorker {
    pub target_building: Entity,
    pub lifespan: u32,
}

const PHANTOM_SPAWN_CHANCE: f64 = 0.1; // 10% chance
const PHANTOM_LIFESPAN: u32 = 500;

pub fn spawn_phantom_workers_system(
    mut commands: Commands,
    query: Query<(&AssignedTo, &GridPosition), Added<Dead>>,
) {
    let mut rng = rand::thread_rng();

    for (assigned, pos) in query.iter() {
        // Must be assigned to a building (not idle/roaming)
        // Ensure it has a chance to spawn
        if rng.gen_bool(PHANTOM_SPAWN_CHANCE) {
            commands.spawn((
                PhantomWorker {
                    target_building: assigned.entity,
                    lifespan: PHANTOM_LIFESPAN,
                },
                *pos, // Spawn at the location of death
            ));
        }
    }
}
const PHANTOM_PRODUCTION_CHANCE: f64 = 0.05; // 5% chance per tick

pub fn phantom_production_system(
    mut commands: Commands,
    mut phantoms: Query<(Entity, &mut PhantomWorker)>,
    buildings: Query<&Building>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut phantom) in phantoms.iter_mut() {
        if phantom.lifespan > 0 {
            phantom.lifespan -= 1;
        } else {
            commands.entity(entity).despawn();
            continue;
        }

        if rng.gen_bool(PHANTOM_PRODUCTION_CHANCE) {
            if let Ok(building) = buildings.get(phantom.target_building) {
                match building.building_type {
                    BuildingType::Farm => {
                        resources.add_food(1.0);
                    }
                    BuildingType::Smelter => {
                        resources.add_metal(1.0);
                    }
                    BuildingType::LumberMill => {
                        resources.add_wood(1.0);
                    }
                    BuildingType::StoneMason => {
                        resources.add_stone(1.0);
                    }
                    _ => {} // Other buildings don't have a direct raw material production like this or produce abstract things
                }
            }
        }
    }
}
const PHANTOM_TERROR_RADIUS: i32 = 3;
const PHANTOM_TERROR_STRESS: f32 = 5.0; // High stress penalty per tick near a phantom

pub fn phantom_terror_system(
    phantoms: Query<&GridPosition, With<PhantomWorker>>,
    mut pops: Query<(&GridPosition, &mut StressTracker), (With<Pop>, Without<Dead>)>,
) {
    if phantoms.is_empty() {
        return;
    }

    // Collect phantom positions
    let mut phantom_positions = Vec::new();
    for pos in phantoms.iter() {
        phantom_positions.push(*pos);
    }

    for (pop_pos, mut stress) in pops.iter_mut() {
        for phantom_pos in &phantom_positions {
            if pop_pos.distance_chebyshev(*phantom_pos) <= PHANTOM_TERROR_RADIUS as u32 {
                stress.accumulated_stress =
                    (stress.accumulated_stress + PHANTOM_TERROR_STRESS).min(100.0);
                break; // Only apply terror once per tick even if near multiple phantoms
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::utility_types::AssignmentType;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_spawn_phantom_workers() {
        let mut world = setup_world();

        let building = world
            .spawn(Building {
                building_type: BuildingType::Farm,
            })
            .id();

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            AssignedTo {
                entity: building,
                assignment_type: AssignmentType::FarmWorker,
            },
            Dead,
        ));

        // Run the system multiple times to bypass rng
        let mut phantom_spawned = false;
        for _ in 0..100 {
            world.run_system_once(spawn_phantom_workers_system).unwrap();
            if world.query::<&PhantomWorker>().iter(&world).count() > 0 {
                phantom_spawned = true;
                break;
            }
        }

        assert!(
            phantom_spawned,
            "A phantom worker should spawn occasionally when an assigned pop dies."
        );
    }

    #[test]
    fn test_phantom_production() {
        let mut world = setup_world();

        let building = world
            .spawn((Building {
                building_type: BuildingType::Farm,
            },))
            .id();

        world.spawn((PhantomWorker {
            target_building: building,
            lifespan: 100,
        },));

        let initial_food = world.resource::<ColonyResources>().food;

        // Run the system multiple times to bypass rng
        let mut produced = false;
        for _ in 0..100 {
            world.run_system_once(phantom_production_system).unwrap();
            let current_food = world.resource::<ColonyResources>().food;
            if current_food > initial_food {
                produced = true;
                break;
            }
        }

        assert!(
            produced,
            "Phantom workers should occasionally generate resources based on the building type."
        );
    }

    #[test]
    fn test_phantom_terror() {
        let mut world = setup_world();

        let building = world
            .spawn((Building {
                building_type: BuildingType::Farm,
            },))
            .id();

        world.spawn((
            PhantomWorker {
                target_building: building,
                lifespan: 100,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(phantom_terror_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(
            stress.accumulated_stress > 0.0,
            "Phantom workers should cause stress to nearby pops."
        );
    }
}
