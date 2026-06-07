use bevy::prelude::*;

#[derive(Component)]
pub struct ScarcityColony;

#[derive(Component)]
pub struct ResourceStorage {
    pub food: u32,
    pub water: u32,
    pub oxygen: u32,
    pub power: u32,
    pub population_demand: u32,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ScarcityJobType {
    RationingBureaucrat,
}

#[derive(Component)]
pub struct JobBoard {
    pub available_jobs: Vec<ScarcityJobType>,
}

#[derive(Component)]
pub struct ScarcityPop;

#[derive(Component)]
pub enum JobAssignment {
    None,
    Active(ScarcityJobType),
}

#[derive(Component)]
pub struct ConsumptionRate {
    pub food_per_tick: f32,
    pub base_food_per_tick: f32,
}

#[derive(Component)]
pub struct GlobalRationingModifier {
    pub reduction_percent: f32,
}

pub fn evaluate_scarcity_system(
    mut colonies: Query<(&ResourceStorage, &mut JobBoard), With<ScarcityColony>>,
) {
    for (storage, mut board) in colonies.iter_mut() {
        if storage.food < storage.population_demand / 2
            || storage.water < storage.population_demand / 2
            || storage.oxygen < storage.population_demand / 2
            || storage.power < storage.population_demand / 2
        {
            if !board
                .available_jobs
                .contains(&ScarcityJobType::RationingBureaucrat)
            {
                // Add 3 bureaucrat jobs as a response to scarcity
                board
                    .available_jobs
                    .push(ScarcityJobType::RationingBureaucrat);
                board
                    .available_jobs
                    .push(ScarcityJobType::RationingBureaucrat);
                board
                    .available_jobs
                    .push(ScarcityJobType::RationingBureaucrat);
            }
        } else if storage.food > storage.population_demand
            && storage.water > storage.population_demand
            && storage.oxygen > storage.population_demand
            && storage.power > storage.population_demand
        {
            board
                .available_jobs
                .retain(|j| *j != ScarcityJobType::RationingBureaucrat);
        }
    }
}

pub fn apply_rationing_buff_system(
    bureaucrats: Query<&JobAssignment, With<ScarcityPop>>,
    mut colonies: Query<&mut GlobalRationingModifier, With<ScarcityColony>>,
    mut consumers: Query<&mut ConsumptionRate, With<ScarcityPop>>,
) {
    let active_bureaucrats = bureaucrats
        .iter()
        .filter(|j| {
            matches!(
                j,
                JobAssignment::Active(ScarcityJobType::RationingBureaucrat)
            )
        })
        .count();

    for mut modifier in colonies.iter_mut() {
        // Each bureaucrat reduces consumption by 5%, capped at 50%
        modifier.reduction_percent = (active_bureaucrats as f32 * 0.05).min(0.50);

        for mut rate in consumers.iter_mut() {
            rate.food_per_tick = rate.base_food_per_tick * (1.0 - modifier.reduction_percent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (evaluate_scarcity_system, apply_rationing_buff_system),
        );
        app
    }

    #[test]
    fn test_bureaucrat_jobs_spawn_during_scarcity() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn((
                ScarcityColony,
                ResourceStorage {
                    food: 10,
                    water: 100,
                    oxygen: 100,
                    power: 100,
                    population_demand: 100,
                }, // Critical scarcity
                JobBoard {
                    available_jobs: vec![],
                },
            ))
            .id();

        app.update();

        let board = app.world().get::<JobBoard>(colony).unwrap();
        assert!(
            board
                .available_jobs
                .contains(&ScarcityJobType::RationingBureaucrat),
            "Scarcity should spawn RationingBureaucrat jobs"
        );
    }

    #[test]
    fn test_bureaucrats_reduce_consumption_rate() {
        let mut app = setup_app();

        let pop_normal = app
            .world_mut()
            .spawn((
                ScarcityPop,
                JobAssignment::None,
                ConsumptionRate {
                    food_per_tick: 2.0,
                    base_food_per_tick: 2.0,
                },
            ))
            .id();

        let _pop_bureaucrat = app
            .world_mut()
            .spawn((
                ScarcityPop,
                JobAssignment::Active(ScarcityJobType::RationingBureaucrat),
                ConsumptionRate {
                    food_per_tick: 2.0,
                    base_food_per_tick: 2.0,
                },
            ))
            .id();

        let colony = app
            .world_mut()
            .spawn((
                ScarcityColony,
                GlobalRationingModifier {
                    reduction_percent: 0.0,
                },
            ))
            .id();

        app.update();

        let modifier = app.world().get::<GlobalRationingModifier>(colony).unwrap();
        assert!(
            modifier.reduction_percent > 0.0,
            "Active bureaucrats should increase the global rationing reduction"
        );

        let normal_rate = app.world().get::<ConsumptionRate>(pop_normal).unwrap();
        assert!(
            normal_rate.food_per_tick < 2.0,
            "Global rationing should reduce individual pop consumption"
        );
    }

    #[test]
    fn test_job_cleanup_when_scarcity_ends() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn((
                ScarcityColony,
                ResourceStorage {
                    food: 150,
                    water: 150,
                    oxygen: 150,
                    power: 150,
                    population_demand: 100,
                }, // Abundance
                JobBoard {
                    available_jobs: vec![ScarcityJobType::RationingBureaucrat],
                },
            ))
            .id();

        app.update();

        let board = app.world().get::<JobBoard>(colony).unwrap();
        assert!(
            !board
                .available_jobs
                .contains(&ScarcityJobType::RationingBureaucrat),
            "Abundance should remove RationingBureaucrat jobs"
        );
    }

    #[test]
    fn test_multiple_resource_scarcity_triggers() {
        let mut app = setup_app();

        // Water scarcity
        let colony = app
            .world_mut()
            .spawn((
                ScarcityColony,
                ResourceStorage {
                    food: 100,
                    water: 10,
                    oxygen: 100,
                    power: 100,
                    population_demand: 100,
                },
                JobBoard {
                    available_jobs: vec![],
                },
            ))
            .id();

        app.update();

        let board = app.world().get::<JobBoard>(colony).unwrap();
        assert!(board
            .available_jobs
            .contains(&ScarcityJobType::RationingBureaucrat));
    }

    #[test]
    fn test_dynamic_base_rate_calculation() {
        let mut app = setup_app();

        let pop_normal = app
            .world_mut()
            .spawn((
                ScarcityPop,
                JobAssignment::None,
                ConsumptionRate {
                    food_per_tick: 5.0,
                    base_food_per_tick: 5.0,
                }, // Higher base rate
            ))
            .id();

        let _pop_bureaucrat = app
            .world_mut()
            .spawn((
                ScarcityPop,
                JobAssignment::Active(ScarcityJobType::RationingBureaucrat),
                ConsumptionRate {
                    food_per_tick: 2.0,
                    base_food_per_tick: 2.0,
                },
            ))
            .id();

        let colony = app
            .world_mut()
            .spawn((
                ScarcityColony,
                GlobalRationingModifier {
                    reduction_percent: 0.0,
                },
            ))
            .id();

        app.update();

        let modifier = app.world().get::<GlobalRationingModifier>(colony).unwrap();
        assert!(modifier.reduction_percent > 0.0);

        let normal_rate = app.world().get::<ConsumptionRate>(pop_normal).unwrap();
        // Base is 5.0, with 5% reduction it should be 4.75
        assert!(
            (normal_rate.food_per_tick - 4.75).abs() < f32::EPSILON,
            "Rate should scale from dynamic base"
        );
    }
}
