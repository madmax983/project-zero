# 264: Hyper-Specialized Evolution

## 1. Overview

"He was born to be a hauler. Literally."

As Pops work the same job for extended periods, their bodies and minds physically adapt to the task. This increases efficiency in their specialized role but inflicts penalties if they are reassigned to different work.

**Mechanic:**
- Pops track **Job Tenure** (ticks worked per Job Type).
- Reaching thresholds triggers **Mutations** (Traits).
- **Specialist Traits** provide bonuses to the specific job and penalties to others.
- **Attrition**: Switching jobs causes "Adaptation Sickness" or stress as the body fights the new role.

**Why:** Encourages long-term job assignment and creates a "Caste System" emergent narrative.

## 2. Dependencies

- `009` — Job System (Job Types)
- `084` — Pop Traits (Trait System)
- `003` — Population Basics (Pops)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/pop/specialization_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Job, JobType, Trait, Traits};
    use crate::layer1::pop::specialization::{JobTenure, update_tenure_system, check_mutation_system};

    #[test]
    fn test_tenure_accumulates_while_working() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: JobType::Miner },
            JobTenure::default(),
        )).id();

        // Run system for 100 ticks
        let mut schedule = Schedule::default();
        schedule.add_systems(update_tenure_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let tenure = world.get::<JobTenure>(pop).unwrap();
        assert_eq!(tenure.get_ticks(JobType::Miner), 100);
    }

    #[test]
    fn test_mutation_trigger_threshold() {
        let mut world = World::new();
        // Spawn pop with tenure just below threshold
        let pop = world.spawn((
            Pop,
            Job { workplace: Entity::PLACEHOLDER, job_type: JobType::Miner },
            JobTenure::with_ticks(JobType::Miner, 9999), // Threshold 10000
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
        let penalty = crate::layer1::pop::traits::get_job_efficiency_modifier(&traits, JobType::Researcher);
        assert!(penalty < 1.0);

        // MoleEyes should be good at mining
        let bonus = crate::layer1::pop::traits::get_job_efficiency_modifier(&traits, JobType::Miner);
        assert!(bonus > 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/pop/specialization.rs

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::pop::{Job, JobType, Traits, Trait};

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
}

pub const MUTATION_THRESHOLD: u64 = 10000; // Configurable

pub fn update_tenure_system(mut query: Query<(&Job, &mut JobTenure)>) {
    for (job, mut tenure) in query.iter_mut() {
        tenure.add_tick(job.job_type);
    }
}

pub fn check_mutation_system(
    mut commands: Commands, // For logging events (future)
    mut query: Query<(&JobTenure, &mut Traits), Changed<JobTenure>>,
) {
    for (tenure, mut traits) in query.iter_mut() {
        // Miner -> MoleEyes
        if tenure.get_ticks(JobType::Miner) >= MUTATION_THRESHOLD && !traits.has(Trait::MoleEyes) {
            traits.add(Trait::MoleEyes);
        }

        // Farmer -> GreenThumb (or equivalent)
        // Add mappings for other jobs
    }
}
```

### 2. Trait Logic

Update `src/layer1/pop/traits.rs`:
- Add `MoleEyes`, `Hunchback` (Hauler), `StaticSkin` (Engineer), `SilverTongue` (Diplomat/Merchant).
- Implement `get_job_efficiency_modifier` to check these traits against `JobType`.

## 5. REFACTOR Phase: Quality & Design

- **Visuals**: Mutations should change the Pop's sprite/description (e.g., "Pale skin, large eyes").
- **Reversion**: Can mutations be cured? Maybe via advanced "Gene Therapy" (Medical).
- **Inheritance**: Integration with `Spec 231` (Memorials) or `Spec 263` (Cadet) - do children inherit mutations? (Future scope).

## 6. Acceptance Criteria

- [ ] `JobTenure` tracks time per job.
- [ ] Working accumulate tenure.
- [ ] Threshold trigger adds specific Traits.
- [ ] Traits provide +/- efficiency based on JobType.
- [ ] Tests pass.

## 7. Technical Guidance

- Use `HashMap` for tenure to support job switching.
- Ensure `JobType` enum is stable (don't rely on integer discriminants if they change).
- Run `check_mutation_system` less frequently (e.g., daily) or on `Changed<JobTenure>` to save perf.

## 8. Questions

*Builder: Should tenure degrade if they stop working the job?*
*Architect:* Yes, specialized tenure decays by 1% per cycle when unassigned from the specialized role.
