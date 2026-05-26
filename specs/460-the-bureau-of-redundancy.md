# 460: The Bureau of Redundancy

## 1. Overview

The "Double-Verification" policy is an edict designed to eradicate workplace accidents through suffocating bureaucracy. When this policy is active, every critical job (like medical surgery, advanced manufacturing, or reactor maintenance) requires *two* Pops to complete. One performs the action, while the other "verifies" it. This drastically reduces the accident or failure rate to 0%, but halves productivity, creating tension during emergencies where a critical job might be delayed simply because a second Pop is not available to verify it.

## 2. Dependencies

- `009` — Job System
- `054` — Colony Edicts

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_critical_job_fails_to_progress_without_verifier_under_redundancy() {
        // Arrange
        let mut world = World::new();
        let mut schedule = ScheduleBuilder::new().build();

        // Setup edict active state
        world.insert_resource(ActiveEdicts(vec![EdictType::DoubleVerification]));

        let pop = world.spawn(PopBundle::default()).id();
        let job = world.spawn(CriticalJob {
            assigned_pop: Some(pop),
            verifier_pop: None,
            progress: 0.0,
        }).id();

        // Act
        world.run_system_once(process_critical_jobs_system).unwrap();

        // Assert
        let updated_job = world.get::<CriticalJob>(job).unwrap();
        assert_eq!(updated_job.progress, 0.0, "Job should not progress without a verifier when edict is active");
    }

    #[test]
    fn test_critical_job_progresses_with_verifier_under_redundancy() {
        // Arrange
        let mut world = World::new();

        world.insert_resource(ActiveEdicts(vec![EdictType::DoubleVerification]));

        let pop1 = world.spawn(PopBundle::default()).id();
        let pop2 = world.spawn(PopBundle::default()).id();

        let job = world.spawn(CriticalJob {
            assigned_pop: Some(pop1),
            verifier_pop: Some(pop2),
            progress: 0.0,
        }).id();

        // Act
        world.run_system_once(process_critical_jobs_system).unwrap();

        // Assert
        let updated_job = world.get::<CriticalJob>(job).unwrap();
        assert!(updated_job.progress > 0.0, "Job should progress with a verifier when edict is active");
    }

    #[test]
    fn test_double_verification_reduces_accident_rate_to_zero() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(ActiveEdicts(vec![EdictType::DoubleVerification]));

        let pop1 = world.spawn(PopBundle::default()).id();
        let pop2 = world.spawn(PopBundle::default()).id();

        let job = world.spawn(CriticalJob {
            assigned_pop: Some(pop1),
            verifier_pop: Some(pop2),
            base_accident_chance: 0.15,
            progress: 0.0,
        }).id();

        // Act
        world.run_system_once(calculate_accident_risk_system).unwrap();

        // Assert
        let updated_job = world.get::<CriticalJob>(job).unwrap();
        assert_eq!(updated_job.calculated_accident_chance, 0.0, "Accident chance must be 0% under Double-Verification");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Process critical jobs, respecting Double-Verification
pub fn process_critical_jobs_system(
    mut query: Query<&mut CriticalJob>,
    edicts: Option<Res<ActiveEdicts>>,
) {
    let requires_verification = edicts
        .map(|e| e.0.contains(&EdictType::DoubleVerification))
        .unwrap_or(false);

    for mut job in query.iter_mut() {
        if requires_verification && job.verifier_pop.is_none() {
            // Cannot progress
            continue;
        }

        job.progress += 10.0; // Basic progression
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Utility AI Integration:** Ensure `process_critical_jobs_system` correctly interfaces with the Utility AI so unemployed Pops evaluate "Verify Critical Job" as a valid and high-priority action when the edict is active.
- **Accident Prevention:** Cleanly separate the base accident calculation logic from the edict override so other safety modifiers can still be applied if the edict is ever revoked.
- **Visual Feedback:** Consider spawning a floating status text icon like "Waiting for Verifier..." when a pop is standing at a machine unable to work.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Edict `Double-Verification` zeroes out accident chances and stalls single-pop critical jobs.

## 7. Technical Guidance

- Integrate tightly with the `JobSystem`'s existing execution schedule.
- You might need to add a `Verifier` or similar property to standard jobs or create a new `CriticalJob` wrapper.
- Ensure only jobs tagged as 'critical' (medical, advanced manufacturing, reactors) are affected by this edict.

## 8. Questions

*Builder: add questions here if spec is unclear.*


*Architect:* I will address these questions as implementation details during the build phase. For the MVP, proceed with the simplest standard approach.

*Builder questions:*
1. The RED phase uses `ScheduleBuilder::new().build()`, but we generally use Bevy's native scheduling. How should I approach this?
   - *Architect:* Use Bevy's native `App::new()` and `app.add_systems(Update, ...)` for the RED phase instead of the conceptual `ScheduleBuilder`. It achieves the same testing goals and aligns with the codebase's standard practices.
