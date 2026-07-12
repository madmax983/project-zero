use crate::layer1::entities::pop::{Pop, PopDied};
use crate::layer1::infrastructure::subconscious_grid::ColonyStress;
use crate::layer1::mind::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct RecentCasualties(pub u32);

#[derive(Component)]
pub struct EchoPlagueActive;

#[derive(Component)]
pub struct DeadPopRecord {
    pub last_action: ActionType,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct EchoPlagueInfected {
    pub target_record: Entity,
    pub last_action: ActionType,
}

pub fn check_echo_plague_outbreak(
    mut commands: Commands,
    query: Query<(Entity, &RecentCasualties, &ColonyStress), Without<EchoPlagueActive>>,
) {
    for (entity, casualties, stress) in query.iter() {
        if casualties.0 >= 10 && stress.average_level >= 75.0 {
            commands.entity(entity).insert(EchoPlagueActive);
        }
    }
}

pub fn track_dead_pop_records(
    mut commands: Commands,
    mut events: EventReader<PopDied>,
    query: Query<&PopAction>,
) {
    for event in events.read() {
        if let Ok(action) = query.get(event.entity) {
            commands.spawn(DeadPopRecord {
                last_action: action.current,
            });
        } else {
            // fallback if pop had no action
            commands.spawn(DeadPopRecord {
                last_action: ActionType::Idle,
            });
        }
    }
}

// Replaces enforce_compulsive_mimicry with a proper Utility AI evaluator
pub fn evaluate_echo_plague_mimicry(
    data: &PopEvalData,
    _buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if let Some(infected) = &data.echo_plague_infected {
        // High priority
        return Some((infected.last_action, 100.0, None));
    }
    None
}

// System to spread the infection naturally, per REFACTOR phase recommendation
pub fn spread_echo_plague_system(
    mut commands: Commands,
    active_colonies: Query<&crate::layer1::infrastructure::subconscious_grid::ColonyStress, With<EchoPlagueActive>>,
    uninfected_pops: Query<(Entity, &crate::layer1::infrastructure::subconscious_grid::ResidentOf), (With<Pop>, Without<EchoPlagueInfected>)>,
    records: Query<(Entity, &DeadPopRecord)>,
) {
    // Only spread if there's an active plague
    if active_colonies.is_empty() {
        return;
    }

    let records_vec: Vec<(Entity, &DeadPopRecord)> = records.iter().collect();
    if records_vec.is_empty() {
        return;
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (pop_entity, resident) in uninfected_pops.iter() {
        // Only infect if the pop's resident colony has the active plague
        if active_colonies.get(resident.0).is_ok() {
            // Small chance to get infected per tick
            if rng.gen_bool(0.001) {
                let (random_record_entity, record) = records_vec[rng.gen_range(0..records_vec.len())];
                commands.entity(pop_entity).insert(EchoPlagueInfected {
                    target_record: random_record_entity,
                    last_action: record.last_action,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Job;
    use crate::layer1::infrastructure::subconscious_grid::Colony;
    use crate::layer1::utility_types::AssignmentType;
    use bevy::prelude::*;

    #[test]
    fn test_echo_plague_spawn_conditions() {
        let mut app = App::new();
        app.add_systems(Update, check_echo_plague_outbreak);

        // Arrange: Setup colony with high casualties and stress
        let colony = app
            .world_mut()
            .spawn((
                Colony,
                RecentCasualties(10), // Threshold exceeded
                ColonyStress {
                    average_level: 80.0,
                }, // Threshold exceeded
            ))
            .id();

        // Act: Run disease spawner
        app.update();

        // Assert: Echo Plague event/status is triggered
        let has_plague = app.world().entity(colony).contains::<EchoPlagueActive>();
        assert!(
            has_plague,
            "Echo Plague should trigger under high casualty and stress conditions"
        );
    }

    #[test]
    fn test_pop_compulsive_mimicry() {
        let mut app = App::new();

        // We simulate the utility AI by testing our evaluator function directly
        let last_action = ActionType::Repair;
        let dead_pop_record = app.world_mut().spawn(DeadPopRecord { last_action }).id();

        let living_pop = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::FarmWorker,
                },
                EchoPlagueInfected {
                    target_record: dead_pop_record,
                    last_action,
                },
            ))
            .id();

        let buffer = UtilityAIBuffer {
            pop_data: vec![],
            // populate enough fields to not panic if needed
            ..Default::default()
        };

        let data = PopEvalData::test_instance();
        let mut test_data = data;
        let infected = *app.world().get::<EchoPlagueInfected>(living_pop).unwrap();
        test_data.echo_plague_infected = Some(infected);

        // Act: Use the evaluator
        let result = evaluate_echo_plague_mimicry(&test_data, &buffer);

        // Assert: Pop's active behavior is overridden to the dead pop's job
        assert!(
            result.is_some(),
            "Evaluator should return a result when infected"
        );
        let (action, utility, _) = result.unwrap();
        assert_eq!(action, ActionType::Repair);
        assert_eq!(utility, 100.0);
    }
}
