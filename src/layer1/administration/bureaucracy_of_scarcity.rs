use bevy::prelude::*;
use crate::layer1::infrastructure::subconscious_grid::Colony;
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::AssignmentType as JobType;

// The RED Phase spec explicitly uses an undefined ResourceStorage, but the codebase
// already has `crate::layer1::resources::ColonyResources` which tracks food, wood, stone, and waste.
// However, the spec asks to generalize `ResourceStorage` to handle Water, Oxygen, or Power
// rather than just Food. Since we must strictly implement what the spec provides for the new feature
// without breaking existing tests, we define our own local ResourceStorage struct for this new specific feature.

#[derive(Component)]
pub struct ResourceStorage {
    pub food: u32,
    pub water: u32,
    pub oxygen: u32,
    pub power: u32,
    pub population_demand: u32,
}

#[derive(Component)]
pub struct JobBoard {
    pub available_jobs: Vec<JobType>,
}

#[derive(Component)]
pub struct GlobalRationingModifier {
    pub reduction_percent: f32,
}

pub fn setup_bureaucracy_of_scarcity(
    mut commands: Commands,
    query: Query<Entity, (With<Colony>, Without<ResourceStorage>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            ResourceStorage {
                food: 100,
                water: 100,
                oxygen: 100,
                power: 100,
                population_demand: 10,
            },
            JobBoard {
                available_jobs: vec![],
            },
            GlobalRationingModifier {
                reduction_percent: 0.0,
            },
        ));
    }
}

pub fn evaluate_scarcity_system(
    mut colonies: Query<(&ResourceStorage, &mut JobBoard), With<Colony>>,
) {
    for (storage, mut board) in colonies.iter_mut() {
        let is_scarce = storage.food < storage.population_demand / 2
            || storage.water < storage.population_demand / 2
            || storage.oxygen < storage.population_demand / 2
            || storage.power < storage.population_demand / 2;

        if is_scarce {
            if !board.available_jobs.contains(&JobType::RationingBureaucrat) {
                // Add 3 bureaucrat jobs as a response to scarcity
                board.available_jobs.push(JobType::RationingBureaucrat);
                board.available_jobs.push(JobType::RationingBureaucrat);
                board.available_jobs.push(JobType::RationingBureaucrat);
            }
        } else {
            // Cleanup phase: if resources are no longer scarce, remove the jobs
            board
                .available_jobs
                .retain(|j| j != &JobType::RationingBureaucrat);
        }
    }
}

pub fn apply_rationing_buff_system(
    bureaucrats: Query<&crate::layer1::pop::Job, With<Pop>>,
    mut colonies: Query<&mut GlobalRationingModifier, With<Colony>>,
) {
    let active_bureaucrats = bureaucrats
        .iter()
        .filter(|j| j.job_type == JobType::RationingBureaucrat)
        .count();

    for mut modifier in colonies.iter_mut() {
        // Each bureaucrat reduces consumption by 5%, capped at 50%
        modifier.reduction_percent = (active_bureaucrats as f32 * 0.05).min(0.50);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                Colony,
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
            board.available_jobs.contains(&JobType::RationingBureaucrat),
            "Scarcity should spawn RationingBureaucrat jobs"
        );
    }

    #[test]
    fn test_bureaucrats_reduce_consumption_rate() {
        let mut app = setup_app();

        let _pop_normal = app
            .world_mut()
            .spawn((
                Pop,
            ))
            .id();

        let _pop_bureaucrat = app
            .world_mut()
            .spawn((
                Pop,
                crate::layer1::pop::Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: JobType::RationingBureaucrat,
                },
            ))
            .id();

        let colony = app
            .world_mut()
            .spawn((
                Colony,
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
    }

    #[test]
    fn test_bureaucrat_jobs_removed_when_no_scarcity() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ResourceStorage {
                    food: 100,
                    water: 100,
                    oxygen: 100,
                    power: 100,
                    population_demand: 100,
                }, // No scarcity
                JobBoard {
                    available_jobs: vec![JobType::RationingBureaucrat],
                },
            ))
            .id();

        app.update();

        let board = app.world().get::<JobBoard>(colony).unwrap();
        assert!(
            !board.available_jobs.contains(&JobType::RationingBureaucrat),
            "No scarcity should remove RationingBureaucrat jobs"
        );
    }
}
