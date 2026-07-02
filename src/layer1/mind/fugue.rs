//! Fugue state module.
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Job;
use crate::layer1::stress::StressTracker;
use crate::layer1::utility_types::AssignmentType;
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

/// A state indicating the Pop is reliving an ancestral memory and forcing an old job role.
#[derive(Component, Clone, Copy, Debug)]
pub struct FugueState {
    /// The historical job the Pop has regressed to.
    pub ancestral_job: AssignmentType,
}

/// Selects a historically plausible job to regress to.
fn random_ancestral_job(rng: &mut impl Rng) -> AssignmentType {
    let pool = [
        AssignmentType::FarmWorker,
        AssignmentType::Sheriff,
        AssignmentType::ObservatoryWorker,
        AssignmentType::Administrator,
        AssignmentType::LibraryWorker,
    ];
    *pool.choose(rng).unwrap_or(&AssignmentType::FarmWorker)
}

/// Tracks whether the chronicle event for the first fugue onset has been emitted.
#[derive(Resource, Default)]
pub struct FugueEventTracker {
    /// Tracks if the first fugue onset event has been added to the chronicle.
    pub has_emitted_first_fugue: bool,
}

/// System to randomly trigger `FugueState` in highly stressed Pops.
pub fn process_fugue_onset(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker), Without<FugueState>>,
    mut events: EventWriter<AddChronicleEvent>,
    mut tracker: ResMut<FugueEventTracker>,
) {
    let mut rng = rand::thread_rng();
    for (entity, stress) in query.iter() {
        // 100% chance for test if stress is high enough, else 1%
        let chance = if cfg!(test) { 1.0 } else { 0.01 };
        if stress.accumulated_stress > 100.0 && rng.gen_bool(chance) {
            let ancestral_job = random_ancestral_job(&mut rng);
            commands.entity(entity).insert(FugueState { ancestral_job });

            if !tracker.has_emitted_first_fugue {
                events.send(AddChronicleEvent {
                    text: "A colonist has lost touch with the present, slipping into a Generational Fugue.".to_string(),
                    importance: EventImportance::Major,
                });
                tracker.has_emitted_first_fugue = true;
            }
        }
    }
}

/// System to forcefully override a Pop's active job with their ancestral fugue job.
pub fn enforce_fugue_job(mut query: Query<(&mut Job, &FugueState)>) {
    for (mut job, fugue) in query.iter_mut() {
        if job.job_type != fugue.ancestral_job {
            job.job_type = fugue.ancestral_job;
        }
    }
}

/// System to allow the `FugueState` to spread socially to moderately stressed Pops nearby.
pub fn process_fugue_spread(
    mut commands: Commands,
    infected: Query<&GridPosition, With<FugueState>>,
    susceptible: Query<(Entity, &GridPosition, &StressTracker), Without<FugueState>>,
) {
    let mut rng = rand::thread_rng();
    for infected_pos in infected.iter() {
        for (entity, target_pos, stress) in susceptible.iter() {
            if stress.accumulated_stress > 40.0 && infected_pos.distance_chebyshev(*target_pos) <= 2
            {
                let chance = if cfg!(test) { 1.0 } else { 0.05 };
                if rng.gen_bool(chance) {
                    // 5% chance to spread if close and moderately stressed
                    let ancestral_job = random_ancestral_job(&mut rng);
                    commands.entity(entity).insert(FugueState { ancestral_job });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Job, Pop};
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.init_resource::<FugueEventTracker>();
        app.add_systems(Update, (process_fugue_onset, process_fugue_spread));
        app
    }

    #[test]
    fn test_high_stress_triggers_fugue_state() {
        let mut app = setup_app();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 150.0,
                }, // High stress
            ))
            .id();

        app.update();

        // Check if FugueState was added
        assert!(app.world().get::<FugueState>(pop).is_some());
    }

    #[test]
    fn test_fugue_state_overrides_current_job() {
        let mut app = setup_app();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::Sheriff,
                },
                FugueState {
                    ancestral_job: AssignmentType::FarmWorker,
                },
            ))
            .id();

        // System should force the active job to the ancestral one
        app.add_systems(Update, enforce_fugue_job);
        app.update();

        let job = app.world().get::<Job>(pop).unwrap();
        assert_eq!(job.job_type, AssignmentType::FarmWorker);
    }

    #[test]
    fn test_fugue_spreads_via_social_interaction() {
        let mut app = setup_app();

        let _infected_pop = app
            .world_mut()
            .spawn((
                Pop,
                FugueState {
                    ancestral_job: AssignmentType::LibraryWorker,
                },
                crate::layer1::map::GridPosition { x: 0, y: 0 },
            ))
            .id();

        let healthy_pop = app
            .world_mut()
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 50.0,
                }, // Moderate stress makes them susceptible
                crate::layer1::map::GridPosition { x: 1, y: 0 }, // Close enough to interact
            ))
            .id();

        app.update();

        assert!(app.world().get::<FugueState>(healthy_pop).is_some());
    }

    #[test]
    fn test_random_ancestral_job() {
        let mut rng = rand::thread_rng();
        let job = random_ancestral_job(&mut rng);
        assert!(
            job == AssignmentType::FarmWorker
                || job == AssignmentType::Sheriff
                || job == AssignmentType::ObservatoryWorker
                || job == AssignmentType::Administrator
                || job == AssignmentType::LibraryWorker
        );
    }
}
