# 716: The Chrono-Phage Contagion

## 1. Overview
**Layer:** 1
**Fantasy:** A disease that steals time from your colonists, leaving them paralyzed in the present while the world moves on.
**Mechanic:** A rare pathogen or localized temporal anomaly infects Pops. Instead of physical damage, infected Pops experience "Stuttering." They randomly freeze in place for extended periods (seconds or minutes of real-time), dropping their current task and consuming Needs at a vastly accelerated rate during the freeze.

## 2. Dependencies
- Needs system (`crate::layer1::needs::Needs`)
- Pop system (`crate::layer1::pop::Pop`)
- Job/Task system (`crate::layer1::jobs::Task` or similar)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Needs, NeedType};
    use crate::layer1::jobs::CurrentTask;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (process_chrono_phage_system, update_stuttering_duration_system));
        app
    }

    #[test]
    fn test_chrono_phage_triggers_stuttering() {
        let mut app = setup_app();

        // Arrange: A pop infected with Chrono-Phage but not currently stuttering
        let entity = app.world_mut().spawn((
            Pop,
            ChronoPhageInfection { severity: 1.0 }, // Guarantees a trigger if evaluated
        )).id();

        // Act
        app.update();

        // Assert: The pop should now have a Stuttering component
        assert!(app.world().entity(entity).contains::<Stuttering>());
    }

    #[test]
    fn test_stuttering_drops_current_task() {
        let mut app = setup_app();

        // Arrange: A pop with a current task that suddenly starts stuttering
        let entity = app.world_mut().spawn((
            Pop,
            CurrentTask { task_id: 1, duration: 10.0 },
            Stuttering { duration_left: 5.0 },
        )).id();

        // Act
        app.update();

        // Assert: The task should be dropped
        assert!(!app.world().entity(entity).contains::<CurrentTask>());
    }

    #[test]
    fn test_stuttering_accelerates_need_consumption() {
        let mut app = setup_app();

        let mut needs = Needs::default();
        needs.set(NeedType::Food, 100.0);

        // Arrange: A pop currently stuttering
        let entity = app.world_mut().spawn((
            Pop,
            needs,
            Stuttering { duration_left: 5.0 }, // 5 ticks left
        )).id();

        // Act: Run one tick
        app.update();

        // Assert: The food need should be consumed at a much higher rate (e.g., -5.0 instead of -1.0)
        let updated_needs = app.world().entity(entity).get::<Needs>().unwrap();
        assert!(updated_needs.get(NeedType::Food) <= 95.0, "Needs should drain significantly faster during stuttering");

        // Assert: Stuttering duration decreases
        let stuttering = app.world().entity(entity).get::<Stuttering>().unwrap();
        assert!(stuttering.duration_left < 5.0);
    }

    #[test]
    fn test_stuttering_ends_when_duration_expires() {
        let mut app = setup_app();

        // Arrange: A pop whose stuttering duration is about to expire
        let entity = app.world_mut().spawn((
            Pop,
            Stuttering { duration_left: 0.1 },
        )).id();

        // Act: Run enough time to clear the stutter
        let mut time = Time::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.5));
        app.insert_resource(time);

        app.update();

        // Assert: The Stuttering component should be removed
        assert!(!app.world().entity(entity).contains::<Stuttering>());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::{Needs, NeedType};
use crate::layer1::jobs::CurrentTask;

#[derive(Component)]
pub struct ChronoPhageInfection {
    pub severity: f32, // 0.0 to 1.0, determines probability of triggering stuttering
}

#[derive(Component)]
pub struct Stuttering {
    pub duration_left: f32, // Time remaining in the freeze state
}

pub fn process_chrono_phage_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ChronoPhageInfection), Without<Stuttering>>,
) {
    for (entity, infection) in query.iter_mut() {
        // Highly simplified: if severity > 0.5, we stutter.
        // In a real implementation, this would use a random roll against severity.
        if infection.severity > 0.5 {
            commands.entity(entity).insert(Stuttering {
                duration_left: 5.0, // Arbitrary starting duration
            });
        }
    }
}

pub fn update_stuttering_duration_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Stuttering, Option<&mut Needs>, Option<&CurrentTask>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut stuttering, needs_opt, task_opt) in query.iter_mut() {
        // Drop current task
        if task_opt.is_some() {
            commands.entity(entity).remove::<CurrentTask>();
        }

        // Accelerate needs
        if let Some(mut needs) = needs_opt {
            let current_food = needs.get(NeedType::Food);
            // Example: drain 10x faster than a normal rate of 0.5 per sec
            needs.set(NeedType::Food, current_food - (5.0 * dt));
        }

        // Update duration
        stuttering.duration_left -= dt;
        if stuttering.duration_left <= 0.0 {
            commands.entity(entity).remove::<Stuttering>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Randomization:** The trigger for `Stuttering` should use a random number generator rolled against the `ChronoPhageInfection.severity` rather than a hardcoded threshold.
- **Duration Variance:** The duration of a stutter should be variable, perhaps influenced by the infection severity.
- **Generalized Needs Drain:** The accelerated need drain should apply to multiple relevant needs (e.g., Hydration, Energy) using a constant multiplier, rather than hardcoding Food.
- **Visuals:** Add an event or marker so rendering systems can apply a "glitch" or "frozen" shader/sprite to stuttering pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/health/chrono_phage.rs` (or equivalent module).
- [ ] Pops with `Stuttering` drop tasks and drain needs rapidly.

## 7. Technical Guidance
- **Module:** Place this in `src/layer1/health/chrono_phage.rs` or similar health/disease module.
- **Integration:** Ensure the system that assigns tasks ignores pops that currently have the `Stuttering` component, otherwise they might get reassigned immediately after dropping a task.
- **Time Dependency:** Use `Res<Time>` carefully to ensure the accelerated needs drain is frame-rate independent.

## 8. Questions
*Builder: add questions here if spec is unclear.*
