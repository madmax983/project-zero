use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::pop::{Job, JobType};
use crate::layer1::traits::{Traits, Trait};

#[derive(Component, Default, Debug, Clone)]
pub struct JobTenure {
    history: HashMap<JobType, u64>,
}

impl JobTenure {
    pub fn get_ticks(&self, job: JobType) -> u64 {
        *self.history.get(&job).unwrap_or(&0)
    }

    pub fn add_tick(&mut self, job: JobType) {
        *self.history.entry(job).or_insert(0) += 1;

        // Decay other jobs by 1% (or just decrement by 1 if > 0 to simulate decay)
        // A full 1% decay per tick would erase tenure almost instantly, so we decrement by 1 instead.
        let mut to_remove = Vec::new();
        for (other_job, ticks) in &mut self.history {
            if *other_job != job && *ticks > 0 {
                *ticks = ticks.saturating_sub(1);
                if *ticks == 0 {
                    to_remove.push(*other_job);
                }
            }
        }
        for j in to_remove {
            self.history.remove(&j);
        }
    }

    pub fn with_ticks(job: JobType, ticks: u64) -> Self {
        let mut t = Self::default();
        t.history.insert(job, ticks);
        t
    }
}

pub const MUTATION_THRESHOLD: u64 = 10000; // Configurable

pub fn update_tenure_system(mut query: Query<(&Job, &mut JobTenure)>) {
    for (job, mut tenure) in query.iter_mut() {
        tenure.add_tick(job.job_type);
    }
}

pub fn check_mutation_system(
    mut query: Query<(&JobTenure, &mut Traits), Changed<JobTenure>>,
) {
    for (tenure, mut traits) in query.iter_mut() {
        if tenure.get_ticks(JobType::LibraryWorker) >= MUTATION_THRESHOLD && !traits.has(Trait::MoleEyes) {
            traits.add(Trait::MoleEyes);
        }
        if tenure.get_ticks(JobType::FarmWorker) >= MUTATION_THRESHOLD && !traits.has(Trait::Hunchback) {
            traits.add(Trait::Hunchback);
        }
        if tenure.get_ticks(JobType::ObservatoryWorker) >= MUTATION_THRESHOLD && !traits.has(Trait::StaticSkin) {
            traits.add(Trait::StaticSkin);
        }
        if tenure.get_ticks(JobType::Administrator) >= MUTATION_THRESHOLD && !traits.has(Trait::SilverTongue) {
            traits.add(Trait::SilverTongue);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Job, JobType};
    use crate::layer1::traits::{Trait, Traits, get_job_efficiency_modifier};
    use crate::layer1::specialization::{JobTenure, update_tenure_system, check_mutation_system};

    #[test]
    fn test_tenure_accumulates_while_working() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: JobType::LibraryWorker },
            JobTenure::default(),
        )).id();

        // Run system for 100 ticks
        let mut schedule = Schedule::default();
        schedule.add_systems(update_tenure_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let tenure = world.get::<JobTenure>(pop).unwrap();
        assert_eq!(tenure.get_ticks(JobType::LibraryWorker), 100);
    }

    #[test]
    fn test_mutation_trigger_threshold() {
        let mut world = World::new();
        // Spawn pop with tenure just below threshold
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: JobType::LibraryWorker },
            JobTenure::with_ticks(JobType::LibraryWorker, 9999), // Threshold 10000
            Traits::default(),
        )).id();

        // Run systems
        let mut schedule = Schedule::default();
        schedule.add_systems((update_tenure_system, check_mutation_system).chain());
        schedule.run(&mut world);

        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::MoleEyes)); // LibraryWorker mutation
    }

    #[test]
    fn test_specialist_penalty_on_wrong_job() {
        let traits = Traits(std::collections::HashSet::from([Trait::MoleEyes]));

        // MoleEyes should be bad at non-library jobs (e.g. FarmWorker)
        let penalty = get_job_efficiency_modifier(&traits, JobType::FarmWorker);
        assert!(penalty < 1.0);

        // MoleEyes should be good at LibraryWorker
        let bonus = get_job_efficiency_modifier(&traits, JobType::LibraryWorker);
        assert!(bonus > 1.0);
    }
}
