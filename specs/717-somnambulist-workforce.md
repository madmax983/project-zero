# 717: The Somnambulist Workforce

## 1. Overview
**Layer:** 1
**Fantasy:** Your colonists are exhausted, but the factory never sleeps. They are working in their dreams, and the results are terrifyingly efficient but completely unguided.
**Mechanic:** When a Pop's Rest need drops to critical levels, they have a chance to enter a "Somnambulist" state instead of collapsing. They continue working at their assigned station with a massive efficiency boost, ignoring all other needs (Hunger, Social). However, their work output is completely randomized—they might flawlessly refine rare alloys, or they might spontaneously dismantle the life support system to build an abstract sculpture out of ducting.

## 2. Dependencies
- Needs system (`crate::layer1::needs::Needs`, specifically `Rest`)
- Job/Task execution (`crate::layer1::jobs::CurrentTask`)
- Production/Work output system (`crate::layer1::production::ProductionOutput`)

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
        app.add_systems(Update, (trigger_somnambulism_system, process_somnambulist_work_system));
        app
    }

    #[test]
    fn test_critically_low_rest_triggers_somnambulism() {
        let mut app = setup_app();

        let mut needs = Needs::default();
        needs.set(NeedType::Rest, 5.0); // Critical low

        let entity = app.world_mut().spawn((
            Pop,
            needs,
            CurrentTask { task_id: 1, duration: 10.0 }, // Currently working
        )).id();

        app.update();

        assert!(app.world().entity(entity).contains::<Somnambulist>());
    }

    #[test]
    fn test_somnambulist_ignores_other_needs_decay() {
        let mut app = setup_app();

        let mut needs = Needs::default();
        needs.set(NeedType::Food, 50.0);
        needs.set(NeedType::Rest, 0.0);

        let entity = app.world_mut().spawn((
            Pop,
            needs,
            Somnambulist { duration_left: 10.0 },
        )).id();

        let mut time = Time::default();
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.insert_resource(time);

        app.update();

        let updated_needs = app.world().entity(entity).get::<Needs>().unwrap();
        // Needs should not have decayed (e.g., normally food drains at 1.0/sec)
        assert_eq!(updated_needs.get(NeedType::Food), 50.0);
    }

    #[test]
    fn test_somnambulist_boosts_efficiency_but_randomizes_output() {
        let mut app = setup_app();

        // This is a complex test: we want to ensure the work amount is higher than normal,
        // and that the output (the task completion or product) is randomized.
        // We simulate a task that normally produces 1 Steel.
        let entity = app.world_mut().spawn((
            Pop,
            Somnambulist { duration_left: 5.0 },
            CurrentTask { task_id: 1, duration: 10.0, base_output: ResourceType::Steel, efficiency: 1.0 },
        )).id();

        app.update();

        let task = app.world().entity(entity).get::<CurrentTask>().unwrap();
        // Efficiency should be significantly boosted (e.g., 5.0 instead of 1.0)
        assert!(task.efficiency >= 5.0);

        // Output might be randomized (e.g., changed from Steel to Sculpture or LifeSupportSabotage)
        // This depends on the specific implementation, but we assert it's changed or flagged as random.
        assert!(task.is_randomized_output);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::{Needs, NeedType};
use crate::layer1::jobs::CurrentTask;

#[derive(Component)]
pub struct Somnambulist {
    pub duration_left: f32,
}

pub fn trigger_somnambulism_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs), (With<Pop>, With<CurrentTask>, Without<Somnambulist>)>,
) {
    for (entity, needs) in query.iter_mut() {
        if needs.get(NeedType::Rest) < 10.0 { // Critical threshold
            commands.entity(entity).insert(Somnambulist {
                duration_left: 60.0, // Arbitrary starting duration
            });
        }
    }
}

pub fn process_somnambulist_work_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Somnambulist, &mut CurrentTask, &mut Needs)>,
) {
    let dt = time.delta_secs();

    for (entity, mut somnambulist, mut task, mut needs) in query.iter_mut() {
        somnambulist.duration_left -= dt;
        if somnambulist.duration_left <= 0.0 {
            commands.entity(entity).remove::<Somnambulist>();
            continue;
        }

        // Freeze needs decay (simple approach: reset them back to previous values or skip decay logic elsewhere)
        // For minimal implementation, we assume a separate system handles decay, and we pause it using a marker.
        // Here, we just boost efficiency and randomize output.

        task.efficiency = 5.0; // Massive boost

        if !task.is_randomized_output {
            // Very simple randomization: flip a boolean flag
            task.is_randomized_output = true;
            // In a real implementation, you'd assign a random output type here (e.g., Sculpture vs Steel).
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Needs Decay Freeze:** Instead of reversing decay, consider a `NeedsFrozen` marker component that the central decay system respects, added/removed with `Somnambulist`.
- **Randomization Tables:** Instead of a simple boolean, implement a loot-table style selection for randomized outcomes, including catastrophic failures (sabotage) and unexpected miracles.
- **Trigger Probability:** Ensure somnambulism isn't guaranteed just by hitting low rest; use a probability check based on total stress or duration of exhaustion.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the somnambulist module.
- [ ] Somnambulist pops do not collapse from low Rest and have massively boosted efficiency but randomized outputs.

## 7. Technical Guidance
- **Module:** Place this in `src/layer1/health/somnambulism.rs` or similar.
- **Integration:** The `calculate_work_amount` system in `src/layer1/execution/general_work.rs` (or equivalent) must check for the `Somnambulist` component to apply the massive efficiency multiplier and handle the randomized output generation (e.g., dispatching a "SabotageEvent" or "MiracleProductionEvent").

## 8. Questions
*Builder: add questions here if spec is unclear.*
