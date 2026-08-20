# 1378: The Martyr's Wake

## 1. Overview
When an elite pop (high prestige) dies while performing a dangerous task (like FarmWorker) under conditions of critically low morale or due to an unaddressed hazard, their death becomes a catalyst for an uncontrollable ideological movement. Their close connections gain a permanent "Grieving Radical" trait, spreading anti-establishment sentiment and inciting wildcat strikes. The player must choose between ruthlessly suppressing the radicals to maintain production or caving to their demands and crippling industrial efficiency.

## 2. Dependencies
- `layer1/entities/pop.rs` (for Pop death events)
- `layer1/social/morale.rs` (for tracking morale)
- `layer1/social/social_stratification.rs` (for social classes like Elite)
- `layer1/mind/utility_types.rs` (for assignment and job types)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::entities::pop::{Pop, PopDied};
    use crate::layer1::social::morale::Morale;
    use crate::layer1::social::social_stratification::SocialClass;
    use crate::layer1::mind::utility_types::AssignmentType;

    #[test]
    fn test_martyrdom_event_triggered_on_elite_dangerous_job_death() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, trigger_martyrdom);
        app.add_event::<PopDied>();
        app.add_event::<MartyrdomEvent>();

        let pop_entity = app.world_mut().spawn((
            Pop,
            SocialClass::Elite,
            Morale { value: 0.1, modifiers: vec![] }, // Critically low morale
            CurrentJob { dangerous: true, job_type: AssignmentType::FarmWorker },
        )).id();

        // Act
        app.world_mut().resource_mut::<Events<PopDied>>().send(
            PopDied { entity: pop_entity, name: "Worker".to_string(), tick: 0 }
        );
        app.update();

        // Assert
        let events = app.world().resource::<Events<MartyrdomEvent>>();
        let mut reader = events.get_cursor();
        let evs: Vec<_> = reader.read(events).collect();
        assert_eq!(evs.len(), 1, "A MartyrdomEvent should be triggered for an Elite pop dying in a dangerous job with low morale.");
    }

    #[test]
    fn test_connections_become_grieving_radicals() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, handle_martyrdom_spread);
        app.add_event::<MartyrdomEvent>();

        let friend_entity = app.world_mut().spawn(Pop).id();

        app.world_mut().resource_mut::<Events<MartyrdomEvent>>().send(
            MartyrdomEvent {
                martyr_job: AssignmentType::FarmWorker,
                connections: vec![friend_entity],
            }
        );

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<GrievingRadical>(friend_entity).is_some(), "Friends of the martyr should become grieving radicals.");
        let radical = app.world().get::<GrievingRadical>(friend_entity).unwrap();
        assert_eq!(radical.boycotted_job, AssignmentType::FarmWorker, "Radicals should refuse to work the job that killed the martyr.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with the full memory and relationship system to dynamically find all close friends and family of the deceased pop.
- Link the `GrievingRadical` component to the Utility AI scoring system so that pops actively avoid the boycotted job and encourage others to strike.
- Add an event hook to the Chronicle to record the martyrdom event as a major historical milestone.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for the new code

## 7. Technical Guidance
- Ensure that the social class threshold for triggering martyrdom is configurable or dynamically adjusts based on colony population size.
- Be careful with how `GrievingRadical` interacts with the job assignment system; pops should simply drop their current job if it matches the boycotted job.

## 8. Questions
*Builder: add questions here if spec is unclear.*
