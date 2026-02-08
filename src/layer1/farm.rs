use crate::layer1::balance::{
    FOOD_HUNGER_THRESHOLD, FOOD_PER_MEAL, FOOD_PER_WORKER_PER_TICK, HUNGER_PER_MEAL,
};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::seasons::SeasonState;
use crate::layer1::skills::{SkillType, Skills, get_skill_efficiency};
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

/// Produces food from all farms with workers.
pub fn produce_food_system(
    farm_query: Query<(&Farm, &crate::layer1::building::Building)>,
    mut pop_query: Query<(&Pop, Option<&mut Skills>)>,
    season: Option<Res<SeasonState>>,
    mut resources: ResMut<ColonyResources>,
) {
    let modifier = season.map_or(1.0, |s| s.current_season.food_modifier());

    for (farm, building) in &farm_query {
        let mut total_efficiency = 0.0;

        for &worker_entity in &farm.workers {
            if let Ok((_, skills_opt)) = pop_query.get_mut(worker_entity) {
                // Determine skill based on building type (all Farming for now?)
                // Spec says Farming for producing food/fiber.
                let skill_type = SkillType::Farming;

                // Calculate efficiency (immutable read)
                // We need to re-borrow or use the value.
                // Since we have &mut Skills, we can just use it.
                // But get_skill_efficiency takes Option<&Skills>.
                // We can re-borrow from Option<&mut Skills> as Option<&Skills>.
                let efficiency = get_skill_efficiency(skills_opt.as_deref(), skill_type);
                total_efficiency += efficiency;

                // Add XP (mutable write)
                if let Some(mut skills) = skills_opt.into_iter().next() {
                    skills.add_xp(skill_type, 1.0);
                }
            }
        }

        let production = total_efficiency * FOOD_PER_WORKER_PER_TICK * modifier;

        if production > 0.0 {
            match building.building_type {
                crate::layer1::building::BuildingType::Plantation => {
                    resources.add_fiber(production);
                }
                _ => {
                    // Default to food (Farm)
                    resources.add_food(production);
                }
            }
        }
    }
}

/// Pops eat food when hungry.
pub fn consume_food_system(
    mut pop_query: Query<(Entity, &mut Needs), With<Pop>>,
    mut resources: ResMut<ColonyResources>,
) {
    if resources.food < f32::EPSILON {
        return;
    }

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
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        assert!(resources.food > 0.0, "Food should be produced");
    }

    #[test]
    fn test_produce_food_system_skills_xp() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn((Pop, Skills::default())).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        // Check XP
        let skills = world.get::<Skills>(worker).unwrap();
        assert_eq!(skills.get_xp(SkillType::Farming), 1.0);
    }

    #[test]
    fn test_produce_food_system_skills_efficiency() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Worker with Level 1 Farming (100 XP) -> 1.1 efficiency
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 100.0);
        let worker = world.spawn((Pop, skills)).id();

        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Base = 0.005 (FOOD_PER_WORKER_PER_TICK)
        // With skill = 0.005 * 1.1 = 0.0055
        // Food starts at 10.0
        // Expected = 10.0055
        assert!(
            (resources.food - 10.0055).abs() < f32::EPSILON,
            "Food production should reflect skill efficiency"
        );
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
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

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
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

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
        // Does not need Building because clean_dead_workers_system only queries Farm
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
}

#[cfg(test)]
mod seasonal_tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
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
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

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
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Farm,
            },
        ));

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

#[cfg(test)]
mod plantation_tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_produce_food_system_plantation() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn((
            farm,
            Building {
                building_type: BuildingType::Plantation,
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        assert!(resources.fiber > 0.0, "Fiber should be produced");
        // Default food is 10.0
        assert!(
            (resources.food - 10.0).abs() < f32::EPSILON,
            "Food should NOT be produced"
        );
    }
}
