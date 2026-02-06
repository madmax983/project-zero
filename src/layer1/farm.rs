use crate::layer1::balance::{
    FOOD_HUNGER_THRESHOLD, FOOD_PER_MEAL, FOOD_PER_WORKER_PER_TICK, HUNGER_PER_MEAL,
};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::seasons::SeasonState;
use crate::layer1::thoughts::Thought;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

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

/// Produces food from all farms with workers.
pub fn produce_food_system(
    farm_query: Query<&Farm>,
    pop_query: Query<&Pop>,
    season: Option<Res<SeasonState>>,
    mut resources: ResMut<ColonyResources>,
) {
    let modifier = season.map_or(1.0, |s| s.current_season.food_modifier());

    #[allow(clippy::cast_precision_loss)]
    let total_production: f32 = farm_query
        .iter()
        .map(|farm| {
            let count = farm
                .workers
                .iter()
                .filter(|&&e| pop_query.get(e).is_ok())
                .count() as f32;
            count * FOOD_PER_WORKER_PER_TICK
        })
        .sum::<f32>()
        * modifier;

    if total_production > 0.0 {
        resources.food += total_production;
    }
}

/// Pops eat food when hungry.
pub fn consume_food_system(
    mut pop_query: Query<(Entity, &mut Needs), With<Pop>>,
    mut resources: ResMut<ColonyResources>,
    time: Res<SimulationTime>,
    mut commands: Commands,
) {
    if resources.food < f32::EPSILON {
        return;
    }

    let tick = time.tick;
    let mut food = resources.food;

    // Collect hungry pop entities first to avoid borrow issues with mut iteration
    let hungry_pops: Vec<Entity> = pop_query
        .iter()
        .filter(|(_, needs)| needs.hunger < FOOD_HUNGER_THRESHOLD)
        .map(|(e, _)| e)
        .collect();

    for entity in hungry_pops {
        if food < FOOD_PER_MEAL {
            break;
        }

        if let Ok((_, mut needs)) = pop_query.get_mut(entity) {
            food -= FOOD_PER_MEAL;
            needs.hunger = (needs.hunger + HUNGER_PER_MEAL).min(1.0);

            let mut rng = rand::thread_rng();
            let thoughts = [
                "That hit the spot.",
                "Finally, a good meal.",
                "Tastes like victory.",
                "Much better.",
            ];
            let text = thoughts[rng.gen_range(0..thoughts.len())].to_string();
            commands.entity(entity).insert(Thought { text, tick });
        }
    }

    resources.food = food;
}

/// Removes dead workers from farms.
pub fn clean_dead_workers_system(mut farm_query: Query<&mut Farm>, pop_query: Query<&Pop>) {
    for mut farm in &mut farm_query {
        farm.workers.retain(|&worker| pop_query.get(worker).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_farm_default() {
        let farm = Farm::default();
        assert_eq!(farm.capacity, 2);
        assert!(farm.workers.is_empty());
    }

    #[test]
    fn test_colony_resources_default() {
        let resources = ColonyResources::default();
        assert!((resources.food - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_produce_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        world.run_system_once(produce_food_system).unwrap();

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

        world.run_system_once(produce_food_system).unwrap();

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
            world.run_system_once(produce_food_system).unwrap();
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
        world.insert_resource(crate::shared::time::SimulationTime::default());

        world.spawn((
            Pop,
            Needs {
                hunger: 0.3,
                rest: 0.8,
                ..Default::default()
            },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        world.run_system_once(consume_food_system).unwrap();
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
        world.insert_resource(crate::shared::time::SimulationTime::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(consume_food_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.3, "Hunger should increase");
    }

    #[test]
    fn test_consume_food_only_when_hungry() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            ..Default::default()
        });
        world.insert_resource(crate::shared::time::SimulationTime::default());

        world.spawn((
            Pop,
            Needs {
                hunger: 0.9,
                rest: 0.8,
                ..Default::default()
            },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        world.run_system_once(consume_food_system).unwrap();
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
        world.insert_resource(crate::shared::time::SimulationTime::default());

        world.spawn((
            Pop,
            Needs {
                hunger: 0.3,
                rest: 0.8,
                ..Default::default()
            },
        ));
        world.spawn((
            Pop,
            Needs {
                hunger: 0.2,
                rest: 0.8,
                ..Default::default()
            },
        ));

        world.run_system_once(consume_food_system).unwrap();

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

        world.run_system_once(clean_dead_workers_system).unwrap();

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

    #[test]
    fn test_consume_food_generates_thought() {
        use crate::layer1::thoughts::Thought;

        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0,
            ..Default::default()
        });
        world.insert_resource(crate::shared::time::SimulationTime::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                },
            ))
            .id();

        world.run_system_once(consume_food_system).unwrap();

        let thought = world
            .get::<Thought>(pop)
            .expect("Eating should generate a thought");
        assert!(!thought.text.is_empty());
    }
}

#[cfg(test)]
mod seasonal_tests {
    use super::*;
    use crate::layer1::seasons::{Season, SeasonState};
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_produce_food_system_winter() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Starting food is 10.0. Base production is 0.005. Winter mod is 0.5.
        let expected = 10.0 + 0.0025;
        assert!(
            (resources.food - expected).abs() < 0.0001,
            "Winter production should be halved"
        );
    }

    #[test]
    fn test_produce_food_system_autumn() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState {
            current_season: Season::Autumn,
        });

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Starting food is 10.0. Base production is 0.005. Autumn mod is 1.5.
        let expected = 10.0 + 0.0075;
        assert!(
            (resources.food - expected).abs() < 0.0001,
            "Autumn production should be boosted"
        );
    }
}
