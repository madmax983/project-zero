# The Generational Fugue (Spec 781)

## 1. Overview
**Layer:** 1 (Colony)
**Fantasy:** A psychological phenomena where colonists lose touch with the present, slipping into the memories of their ancestors due to the long-term effects of cryo-sleep or isolation.
**Mechanic:** Pops can develop a "Fugue State" trait under high stress. While in this state, they will attempt to perform jobs or tasks that were relevant to their great-grandparents (e.g., trying to farm in a heavy industrial sector, or building ancestral shelters in the middle of a modern arcology). This state spreads if others talk to them for too long.
**Emergence:** A vital plasma reactor shuts down because the lead engineer is found in the corner, desperately trying to churn non-existent butter or weave baskets out of fiber-optic cables, leading to cascading power failures.
**Tension:** Do you isolate these "historians" to protect your modern infrastructure, or try to cure them, risking the spread of the fugue to your medical staff?

## 2. Dependencies
- `layer1::pop::Pop` and `PopState` logic.
- `layer1::stress::StressTracker` to trigger the condition.
- `layer1::social::interaction` for spreading the fugue state.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::job::JobRole;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (process_fugue_onset, process_fugue_spread));
        app
    }

    #[test]
    fn test_high_stress_triggers_fugue_state() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop::default(),
            StressTracker { current: 95.0, ..Default::default() }, // High stress
        )).id();

        app.update();

        // Check if FugueState was added
        assert!(app.world().get::<FugueState>(pop).is_some());
    }

    #[test]
    fn test_fugue_state_overrides_current_job() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop::default(),
            JobRole::Engineer,
            FugueState { ancestral_job: JobRole::Farmer },
        )).id();

        // System should force the active job to the ancestral one
        app.add_systems(Update, enforce_fugue_job);
        app.update();

        let job = app.world().get::<JobRole>(pop).unwrap();
        assert_eq!(*job, JobRole::Farmer);
    }

    #[test]
    fn test_fugue_spreads_via_social_interaction() {
        let mut app = setup_app();

        let infected_pop = app.world_mut().spawn((
            Pop::default(),
            FugueState { ancestral_job: JobRole::Miner },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let healthy_pop = app.world_mut().spawn((
            Pop::default(),
            StressTracker { current: 50.0, ..Default::default() }, // Moderate stress makes them susceptible
            Transform::from_xyz(1.0, 0.0, 0.0), // Close enough to interact
        )).id();

        app.update();

        assert!(app.world().get::<FugueState>(healthy_pop).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use bevy_transform::prelude::Transform;
use crate::layer1::stress::StressTracker;
use crate::layer1::job::JobRole;
use rand::Rng;

#[derive(Component, Clone, Copy)]
pub struct FugueState {
    pub ancestral_job: JobRole,
}

pub fn process_fugue_onset(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker), Without<FugueState>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, stress) in query.iter() {
        if stress.current > 90.0 && rng.gen_bool(0.01) { // 1% chance per tick when highly stressed
            commands.entity(entity).insert(FugueState {
                ancestral_job: JobRole::Farmer, // Hardcoded for MVP, should be randomized or historically driven
            });
        }
    }
}

pub fn enforce_fugue_job(
    mut query: Query<(&mut JobRole, &FugueState)>,
) {
    for (mut job, fugue) in query.iter_mut() {
        if *job != fugue.ancestral_job {
            *job = fugue.ancestral_job;
        }
    }
}

pub fn process_fugue_spread(
    mut commands: Commands,
    infected: Query<&Transform, With<FugueState>>,
    susceptible: Query<(Entity, &Transform, &StressTracker), Without<FugueState>>,
) {
    let mut rng = rand::thread_rng();
    for infected_transform in infected.iter() {
        for (entity, target_transform, stress) in susceptible.iter() {
            if stress.current > 40.0 && infected_transform.translation.distance(target_transform.translation) < 2.0 {
                if rng.gen_bool(0.05) { // 5% chance to spread if close and moderately stressed
                    commands.entity(entity).insert(FugueState {
                        ancestral_job: JobRole::Farmer, // Simplified for MVP
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Ancestral Job Selection**: Instead of hardcoding `JobRole::Farmer`, the `ancestral_job` should be drawn from a pool of historically plausible jobs or actual data if `layer1::memory` tracks lineage.
- **Curing the Fugue**: Needs a medical or psychological intervention mechanic (e.g., a "Reality Anchor" building or therapy sessions by a Doctor pop).
- **Chronicle Hook**: When the first Pop enters a Fugue State, trigger an `AddChronicleEvent` to warn the player of the psychological breakdown.

## 6. Acceptance Criteria
- [ ] `FugueState` component created.
- [ ] High stress triggers the onset of `FugueState`.
- [ ] Pops in a `FugueState` forcefully adopt their `ancestral_job`.
- [ ] `FugueState` can spread to nearby stressed Pops.
- [ ] Test coverage >85%.

## 7. Technical Guidance
- Implement in `src/layer1/mind/fugue.rs`.
- Ensure `enforce_fugue_job` runs after standard job assignment systems to guarantee the override.

## 8. Questions
*Builder: add questions here if spec is unclear.*
