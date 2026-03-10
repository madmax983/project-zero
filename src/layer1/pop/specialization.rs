use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::pop::{Job, JobType, Traits};
use crate::layer1::Trait;

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
    }

    pub fn with_ticks(job: JobType, ticks: u64) -> Self {
        let mut t = Self::default();
        t.history.insert(job, ticks);
        t
    }

    pub fn apply_degradation(&mut self, current_job: Option<JobType>) {
        // Iterate through history, skipping current_job
        for (job, ticks) in self.history.iter_mut() {
            if Some(*job) != current_job {
                if *ticks > 0 {
                    // Reduce by 1% per cycle. Ensure it doesn't drop below 0
                    // Since it's integer math, we use max(1, ticks / 100) to ensure at least 1 tick decays
                    let decay = std::cmp::max(1, *ticks / 100);
                    *ticks = ticks.saturating_sub(decay);
                }
            }
        }
    }
}

pub const MUTATION_THRESHOLD: u64 = 10000;

pub fn update_tenure_system(mut query: Query<(Option<&Job>, &mut JobTenure)>) {
    for (job_opt, mut tenure) in query.iter_mut() {
        if let Some(job) = job_opt {
            tenure.add_tick(job.job_type);
            tenure.apply_degradation(Some(job.job_type));
        } else {
            tenure.apply_degradation(None);
        }
    }
}

pub fn check_mutation_system(
    mut query: Query<(&JobTenure, &mut Traits), Changed<JobTenure>>,
) {
    for (tenure, mut traits) in query.iter_mut() {
        if tenure.get_ticks(JobType::FarmWorker) >= MUTATION_THRESHOLD && !traits.has(Trait::MoleEyes) {
            traits.add(Trait::MoleEyes);
        }

        if tenure.get_ticks(JobType::LibraryWorker) >= MUTATION_THRESHOLD && !traits.has(Trait::StaticSkin) {
            traits.add(Trait::StaticSkin);
        }

        if tenure.get_ticks(JobType::Administrator) >= MUTATION_THRESHOLD && !traits.has(Trait::SilverTongue) {
            traits.add(Trait::SilverTongue);
        }
    }
}
