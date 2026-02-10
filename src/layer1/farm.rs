use crate::layer1::balance::{
    FOOD_HUNGER_THRESHOLD, FOOD_PER_MEAL, FOOD_PER_WORKER_PER_TICK, HUNGER_PER_MEAL,
};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::seasons::SeasonState;
use crate::layer1::skills::{SkillType, Skills, get_skill_efficiency};
use crate::layer1::utility_ai::types::{ActionType, PopAction};
use crate::layer1::GridPosition;
use bevy_ecs::prelude::*;

/// Farm component - produces food when worked.
#[derive(Component)]
pub struct Farm {
    /// Maximum number of workers.
    pub capacity: usize,
    /// List of workers assigned to this farm.
    /// **Legacy**: Used for UI/Capacity checks, but production uses `PopAction`.
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

/// Produces food from all farms with active workers.
pub fn produce_food_system(
    farm_query: Query<(&crate::layer1::building::Building, &GridPosition), With<Farm>>,
    mut pop_query: Query<
        (Entity, &GridPosition, &PopAction, Option<&mut Skills>),
        With<Pop>,
    >,
    season: Option<Res<SeasonState>>,
    mut resources: ResMut<ColonyResources>,
) {
    let modifier = season.map_or(1.0, |s| s.current_season.food_modifier());

    // Collect farmers to avoid borrowing issues
    // We need mutable access to Skills, so we can't collect references easily if we want to iterate multiple times?
    // Actually, we can iterate pop_query mutably ONCE.
    // But we need to match them to farms.
    // Since we iterate farms, we would need random access to pops.

    // Better approach: Iterate ALL farmers, find which farm they are at (if any), and produce.
    // This avoids O(F*W) if we have a map.
    // But we don't have a map.
    // However, we can query buildings by position? No.
    // We can collect farms into a Map<GridPosition, BuildingType>.

    let farm_map: std::collections::HashMap<GridPosition, crate::layer1::building::BuildingType> = farm_query
        .iter()
        .map(|(b, p)| (*p, b.building_type))
        .collect();

    for (_, pos, action, skills_opt) in &mut pop_query {
        if action.current != ActionType::Farm {
            continue;
        }

        if let Some(building_type) = farm_map.get(pos) {
            let skill_type = SkillType::Farming;

            // Calculate efficiency
            let efficiency = get_skill_efficiency(skills_opt.as_deref(), skill_type);

            // Add XP
            if let Some(mut skills) = skills_opt {
                skills.add_xp(skill_type, 1.0);
            }

            let production = efficiency * FOOD_PER_WORKER_PER_TICK * modifier;

            if production > 0.0 {
                match building_type {
                    crate::layer1::building::BuildingType::Plantation => {
                        resources.add_fiber(production);
                    }
                    _ => {
                        resources.add_food(production);
                    }
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
    fn test_produce_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn worker at farm doing Farm action
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
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

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let worker = world.spawn((
            Pop,
            Skills::default(),
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        )).id();

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

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
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

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Two workers
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        assert!(
            resources.food >= 0.009,
            "Two workers should produce more food"
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
}
