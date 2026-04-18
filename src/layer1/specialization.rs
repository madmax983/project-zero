use bevy_ecs::prelude::*;
use crate::layer1::pop::Job;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_types::AssignmentType;
use bevy::utils::HashMap;

#[derive(Component, Default, Debug, Clone)]
pub struct JobTenure {
    history: HashMap<AssignmentType, u64>,
}

impl JobTenure {
    pub fn get_ticks(&self, job: AssignmentType) -> u64 {
        *self.history.get(&job).unwrap_or(&0)
    }

    pub fn add_tick(&mut self, job: AssignmentType) {
        *self.history.entry(job).or_insert(0) += 1;
    }

    pub fn with_ticks(job: AssignmentType, ticks: u64) -> Self {
        let mut t = Self::default();
        t.history.insert(job, ticks);
        t
    }
}

pub const MUTATION_THRESHOLD: u64 = 10000;

/// ⚡ Bolt Optimization: Switched to `bevy::utils::HashMap` and eliminated per-frame `Vec` allocation in `update_tenure_system`.
pub fn update_tenure_system(mut query: Query<(&Job, &mut JobTenure)>) {
    for (job, mut tenure) in query.iter_mut() {
        tenure.add_tick(job.job_type);

        // Architect: specialized tenure decays by 1% per cycle when unassigned
        for (job_type, ticks) in tenure.history.iter_mut() {
            if *job_type != job.job_type && *ticks > 0 {
                *ticks = (*ticks as f64 * 0.99) as u64;
            }
        }
    }
}

pub fn check_mutation_system(
    mut query: Query<(&JobTenure, &mut Traits), Changed<JobTenure>>,
) {
    for (tenure, mut traits) in query.iter_mut() {
        if tenure.get_ticks(AssignmentType::FarmWorker) >= MUTATION_THRESHOLD && !traits.has(Trait::GreenThumb) {
            traits.add(Trait::GreenThumb);
        }
        if tenure.get_ticks(AssignmentType::Administrator) >= MUTATION_THRESHOLD && !traits.has(Trait::SilverTongue) {
            traits.add(Trait::SilverTongue);
        }
        // TODO: add other job traits once AssignmentType supports hauling and engineering
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_tenure_accumulates_while_working() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::FarmWorker },
            JobTenure::default(),
        )).id();

        // Run system for 100 ticks
        let mut schedule = Schedule::default();
        schedule.add_systems(update_tenure_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let tenure = world.get::<JobTenure>(pop).unwrap();
        assert_eq!(tenure.get_ticks(AssignmentType::FarmWorker), 100);
    }

    #[test]
    fn test_mutation_trigger_threshold() {
        let mut world = World::new();
        // Spawn pop with tenure just below threshold
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::FarmWorker },
            JobTenure::with_ticks(AssignmentType::FarmWorker, 9999), // Threshold 10000
            Traits::default(),
        )).id();

        // Run systems
        let mut schedule = Schedule::default();
        schedule.add_systems((update_tenure_system, check_mutation_system).chain());
        schedule.run(&mut world);

        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::GreenThumb)); // FarmWorker mutation
    }

    #[test]
    fn test_tenure_decays_while_working_different_job() {
        let mut world = World::new();

        let job_tenure = JobTenure::with_ticks(AssignmentType::FarmWorker, 100);
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::Administrator },
            job_tenure,
        )).id();

        // Run system for 1 tick
        let mut schedule = Schedule::default();
        schedule.add_systems(update_tenure_system);

        schedule.run(&mut world);

        let tenure = world.get::<JobTenure>(pop).unwrap();
        // 100 * 0.99 = 99
        assert_eq!(tenure.get_ticks(AssignmentType::FarmWorker), 99);
        assert_eq!(tenure.get_ticks(AssignmentType::Administrator), 1);
    }

    #[test]
    fn test_specialist_penalty_on_wrong_job() {
        // This relies on the work speed calculation system using traits
        let mut traits = Traits::default();
        traits.add(Trait::GreenThumb);

        // GreenThumb should be bad at non-farming jobs (e.g. Administrator)
        let penalty = crate::layer1::traits::get_job_efficiency_modifier(&traits, AssignmentType::Administrator);
        assert!(penalty < 1.0);

        // GreenThumb should be good at farming
        let bonus = crate::layer1::traits::get_job_efficiency_modifier(&traits, AssignmentType::FarmWorker);
        assert!(bonus > 1.0);
    }
}
