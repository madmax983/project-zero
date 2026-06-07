use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::entities::pop::{Job, Pop};
use crate::layer1::mind::utility_types::AssignmentType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ScarcityBureaucracy {
    /// Number of rationing bureaucrats needed.
    pub target_bureaucrats: usize,
}

#[derive(Component, Resource, Default)]
pub struct GlobalRationingModifier {
    pub reduction_percent: f32,
}

pub fn evaluate_scarcity_system(
    mut commands: Commands,
    resources: Option<Res<ColonyResources>>,
    mut bureaucracy_query: Query<(Entity, &mut ScarcityBureaucracy)>,
) {
    if let Some(res) = resources {
        let is_scarce = res.food < 50.0; // Using 50.0 as a threshold for "critical failure" since we don't have population_demand globally tracked by default.

        if is_scarce {
            if let Ok((_, mut bureaucracy)) = bureaucracy_query.get_single_mut() {
                bureaucracy.target_bureaucrats = 3;
            } else {
                commands.spawn(ScarcityBureaucracy {
                    target_bureaucrats: 3,
                });
            }
        } else if let Ok((entity, mut bureaucracy)) = bureaucracy_query.get_single_mut() {
            bureaucracy.target_bureaucrats = 0;
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_bureaucrat_jobs_system(
    mut commands: Commands,
    bureaucracy_query: Query<&ScarcityBureaucracy>,
    bureaucrats: Query<(Entity, &Job), With<Pop>>,
) {
    if bureaucracy_query.is_empty() {
        for (entity, job) in bureaucrats.iter() {
            if job.job_type == AssignmentType::RationingBureaucrat {
                commands.entity(entity).remove::<Job>();
            }
        }
    }
}

pub fn apply_rationing_buff_system(
    bureaucrats: Query<&Job, With<Pop>>,
    modifier: Option<ResMut<GlobalRationingModifier>>,
) {
    let active_bureaucrats = bureaucrats
        .iter()
        .filter(|j| j.job_type == AssignmentType::RationingBureaucrat)
        .count();

    let reduction = (active_bureaucrats as f32 * 0.05).min(0.50);
    if let Some(mut mod_res) = modifier {
        mod_res.reduction_percent = reduction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.add_systems(
            bevy_app::Update,
            (
                evaluate_scarcity_system,
                apply_rationing_buff_system,
                cleanup_bureaucrat_jobs_system,
            ),
        );
        app.insert_resource(GlobalRationingModifier::default());
        app
    }

    #[test]
    fn test_evaluate_scarcity_spawns_bureaucracy() {
        let mut app = setup_app();
        app.world_mut().insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });
        app.update();
        let count = app
            .world_mut()
            .query::<&ScarcityBureaucracy>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_bureaucrats_reduce_consumption_rate() {
        let mut app = setup_app();
        app.world_mut().spawn(ScarcityBureaucracy {
            target_bureaucrats: 3,
        });
        app.world_mut().spawn((
            Pop,
            Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::RationingBureaucrat,
            },
        ));
        app.update();
        let modifier = app.world().resource::<GlobalRationingModifier>();
        assert!(modifier.reduction_percent > 0.0);
        assert!((modifier.reduction_percent - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cleanup_bureaucrat_jobs_system_removes_jobs_when_no_bureaucracy() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::RationingBureaucrat,
                },
            ))
            .id();
        app.update();
        assert!(app.world().get::<Job>(entity).is_none());
    }

    #[test]
    fn test_cleanup_bureaucrat_jobs_system_keeps_jobs_when_bureaucracy_exists() {
        let mut app = setup_app();
        app.world_mut().spawn(ScarcityBureaucracy {
            target_bureaucrats: 3,
        });
        let entity = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::RationingBureaucrat,
                },
            ))
            .id();
        app.update();
        assert!(app.world().get::<Job>(entity).is_some());
    }
}
