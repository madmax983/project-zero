#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Job, JobType, Traits};
    use crate::layer1::Trait;
    use crate::layer1::pop::specialization::{JobTenure, update_tenure_system, check_mutation_system};

    #[test]
    fn test_tenure_accumulates_while_working() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: JobType::FarmWorker },
            JobTenure::default(),
        )).id();

        // Run system for 100 ticks
        let mut schedule = Schedule::default();
        schedule.add_systems(update_tenure_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let tenure = world.get::<JobTenure>(pop).unwrap();
        assert_eq!(tenure.get_ticks(JobType::FarmWorker), 100);
    }

    #[test]
    fn test_mutation_trigger_threshold() {
        let mut world = World::new();
        // Spawn pop with tenure just below threshold
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: JobType::FarmWorker },
            JobTenure::with_ticks(JobType::FarmWorker, 9999), // Threshold 10000
            Traits::default(),
        )).id();

        // Run systems
        let mut schedule = Schedule::default();
        schedule.add_systems((update_tenure_system, check_mutation_system).chain());
        schedule.run(&mut world);

        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::MoleEyes)); // Miner mutation
    }

    #[test]
    fn test_specialist_penalty_on_wrong_job() {
        // This relies on the work speed calculation system using traits
        let traits = Traits(std::collections::HashSet::from([Trait::MoleEyes]));

        // MoleEyes should be bad at non-mining jobs (e.g. Researcher)
        let penalty = crate::layer1::traits::get_job_efficiency_modifier(&traits, JobType::LibraryWorker);
        assert!(penalty < 1.0);

        // MoleEyes should be good at mining
        let bonus = crate::layer1::traits::get_job_efficiency_modifier(&traits, JobType::FarmWorker);
        assert!(bonus > 1.0);
    }
}
