use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

/// Farm component - produces food when worked.
#[derive(Component)]
pub struct Farm {
    /// Maximum number of workers.
    pub capacity: usize,
    /// List of workers assigned to this farm.
    pub workers: Vec<Entity>,
}

impl Default for Farm {
    fn default() -> Self {
        Self {
            capacity: 2,
            workers: Vec::new(),
        }
    }
}

const FOOD_PER_WORKER_PER_TICK: f32 = 0.005;
const FOOD_HUNGER_THRESHOLD: f32 = 0.7; // Eat when below this
const FOOD_PER_MEAL: f32 = 0.1; // Food consumed per meal
const HUNGER_PER_MEAL: f32 = 0.3; // Hunger restored per meal

/// Produces food from all farms with workers.
pub fn produce_food_system(world: &mut World) {
    let mut total_production = 0.0;

    // Use a scope to drop the borrow on world from the query
    let production_from_farms: f32 = {
        let mut query = world.query::<&Farm>();
        // We collect the workers to check validity later to avoid nested borrow issues if any,
        // although shared-shared should be fine. But to be safe and consistent with cleanup pattern:
        // Actually, shared-shared is fine.
        query
            .iter(world)
            .map(|farm| {
                #[allow(clippy::cast_precision_loss)]
                let count = farm
                    .workers
                    .iter()
                    .filter(|&&e| world.get_entity(e).is_ok())
                    .count() as f32;
                count * FOOD_PER_WORKER_PER_TICK
            })
            .sum()
    };

    total_production += production_from_farms;

    if total_production > 0.0 {
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.food += total_production;
    }
}

/// Pops eat food when hungry.
pub fn consume_food_system(world: &mut World) {
    // Optimization: Check if we have any food before querying
    if world.resource::<ColonyResources>().food < f32::EPSILON {
        return;
    }

    let hungry_pops: Vec<Entity> = world
        .query_filtered::<(Entity, &Needs), With<Pop>>()
        .iter(world)
        .filter(|(_, needs)| needs.hunger < FOOD_HUNGER_THRESHOLD)
        .map(|(e, _)| e)
        .collect();

    let meal_cost = ColonyResources {
        food: FOOD_PER_MEAL,
        ..Default::default()
    };

    for entity in hungry_pops {
        // Verify entity has Needs (read-only check)
        let has_needs = world.get::<Needs>(entity).is_some();

        if has_needs {
            if world.resource_mut::<ColonyResources>().try_deduct(&meal_cost) {
                // Re-acquire mutable access to update Needs
                // Safe because we have exclusive world access and try_deduct doesn't remove entities
                if let Some(mut needs) = world.get_mut::<Needs>(entity) {
                    needs.hunger = (needs.hunger + HUNGER_PER_MEAL).min(1.0);
                }
            } else {
                // Out of food
                break;
            }
        }
    }
}

/// Removes dead workers from farms.
pub fn clean_dead_workers_system(world: &mut World) {
    // Collect all farm entities first to avoid keeping a borrow on the world
    let farm_entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Farm>>()
        .iter(world)
        .collect();

    for farm_entity in farm_entities {
        // Read the farm to get workers (clone the vec to release borrow)
        let workers = if let Some(farm) = world.get::<Farm>(farm_entity) {
            farm.workers.clone()
        } else {
            continue;
        };

        // Identify dead workers
        let dead_workers: Vec<Entity> = workers
            .iter()
            .filter(|&&worker| world.get_entity(worker).is_err())
            .copied()
            .collect();

        // If there are dead workers, remove them
        if dead_workers.is_empty() {
            continue;
        }

        if let Some(mut farm) = world.get_mut::<Farm>(farm_entity) {
            farm.workers.retain(|w| !dead_workers.contains(w));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_farm_default() {
        let farm = Farm::default();
        assert_eq!(farm.capacity, 2);
        assert!(farm.workers.is_empty());
    }

    #[test]
    fn test_colony_resources_default() {
        let resources = ColonyResources::default();
        assert!(resources.food.abs() < f32::EPSILON);
    }

    #[test]
    fn test_produce_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert!(resources.food > 0.0, "Food should be produced");
    }

    #[test]
    fn test_produce_food_multiple_workers() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker1 = world.spawn(Pop).id();
        let worker2 = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker1);
        farm.workers.push(worker2);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert!(
            resources.food >= 0.009,
            "Two workers should produce more food"
        );
    }

    #[test]
    fn test_produce_food_multiple_ticks() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        for _ in 0..600 {
            produce_food_system(&mut world);
        }

        let resources = world.resource::<ColonyResources>();
        println!("Food accumulated: {}", resources.food);
        // Use epsilon for float comparison
        assert!(
            resources.food >= 2.99,
            "600 ticks should accumulate ~3.0 food"
        );
    }

    #[test]
    fn test_consume_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            ..Default::default()
        });

        world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.8,
            },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        consume_food_system(&mut world);
        let food_after = world.resource::<ColonyResources>().food;

        assert!(food_after < food_before, "Food should be consumed");
    }

    #[test]
    fn test_consume_food_restores_hunger() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.8,
                },
            ))
            .id();

        consume_food_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.5, "Hunger should increase");
    }

    #[test]
    fn test_consume_food_only_when_hungry() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            ..Default::default()
        });

        world.spawn((
            Pop,
            Needs {
                hunger: 0.9,
                rest: 0.8,
            },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        consume_food_system(&mut world);
        let food_after = world.resource::<ColonyResources>().food;

        assert!(
            (food_after - food_before).abs() < f32::EPSILON,
            "High hunger pop should not eat"
        );
    }

    #[test]
    fn test_consume_food_stops_when_depleted() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 0.05,
            ..Default::default()
        }); // Less than meal cost

        world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.8,
            },
        ));
        world.spawn((
            Pop,
            Needs {
                hunger: 0.4,
                rest: 0.8,
            },
        ));

        consume_food_system(&mut world);

        // At most one pop should eat
        let resources = world.resource::<ColonyResources>();
        assert!(resources.food >= 0.0, "Food should not go negative");
    }

    #[test]
    fn test_clean_dead_workers_system() {
        let mut world = World::new();

        let worker1 = world.spawn(Pop).id();
        let worker2 = world.spawn(Pop).id();

        let mut farm = Farm::default();
        farm.workers.push(worker1);
        farm.workers.push(worker2);
        let farm_entity = world.spawn(farm).id();

        world.despawn(worker1);

        clean_dead_workers_system(&mut world);

        let farm = world.get::<Farm>(farm_entity).unwrap();
        assert_eq!(farm.workers.len(), 1);
        assert_eq!(farm.workers[0], worker2);
    }

    #[test]
    fn test_farm_component_with_building() {
        let mut world = World::new();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
            Farm::default(),
        ));

        let count = world.query::<(&Building, &Farm)>().iter(&world).count();
        assert_eq!(count, 1);
    }
}
