# Spec 358: Stress Breakdowns

## 1. Overview
When a Pop's Stress exceeds a critical threshold, they shouldn't just be unhappy—they should break down. A breakdown is a state determined by the Pop's traits (e.g., Pyromania, Vandalism). It forces them out of their normal job cycle and triggers destructive behaviors, requiring player intervention.

## 2. Dependencies
- `004-pop-entity.md` (for Pops)
- `005-pop-needs.md` (for Stress tracking)
- `084-pop-traits.md` (for Trait components)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::traits::Trait;
    use crate::layer1::grid::GridPosition;

    #[test]
    fn test_pop_enters_breakdown_when_stress_maxed() {
        let mut app = App::new();
        app.add_systems(Update, check_stress_breakdown_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { stress: 100.0, ..default() },
            Trait::Volatile,
        )).id();

        app.update();

        // Pop should now have a StressBreakdown component
        let breakdown = app.world().get::<StressBreakdown>(pop);
        assert!(breakdown.is_some());
        assert_eq!(breakdown.unwrap().breakdown_type, BreakdownType::Pyromania);
    }

    #[test]
    fn test_breakdown_cancels_current_job() {
        let mut app = App::new();
        app.add_systems(Update, apply_breakdown_effects_system);

        let pop = app.world_mut().spawn((
            Pop,
            StressBreakdown { breakdown_type: BreakdownType::Catatonia, duration: 100 },
            ActiveJob::Mining,
        )).id();

        app.update();

        // ActiveJob should be removed during a breakdown
        assert!(app.world().get::<ActiveJob>(pop).is_none());
    }

    #[test]
    fn test_breakdown_duration_decreases() {
        let mut app = App::new();
        app.add_systems(Update, tick_breakdown_duration_system);

        let pop = app.world_mut().spawn((
            Pop,
            StressBreakdown { breakdown_type: BreakdownType::Catatonia, duration: 100 },
        )).id();

        app.update();

        // Duration should decrement
        let breakdown = app.world().get::<StressBreakdown>(pop).unwrap();
        assert_eq!(breakdown.duration, 99);
    }

    #[test]
    fn test_breakdown_removed_when_duration_zero() {
        let mut app = App::new();
        app.add_systems(Update, tick_breakdown_duration_system);

        let pop = app.world_mut().spawn((
            Pop,
            StressBreakdown { breakdown_type: BreakdownType::Catatonia, duration: 0 },
        )).id();

        app.update();

        // Breakdown should be removed
        assert!(app.world().get::<StressBreakdown>(pop).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::traits::Trait;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BreakdownType {
    Pyromania,
    Catatonia,
    Vandalism,
}

#[derive(Component)]
pub struct StressBreakdown {
    pub breakdown_type: BreakdownType,
    pub duration: u32,
}

#[derive(Component)]
pub enum ActiveJob {
    Mining,
    Farming,
}

pub fn check_stress_breakdown_system(
    mut commands: Commands,
    query: Query<(Entity, &Needs, Option<&Trait>), (With<Pop>, Without<StressBreakdown>)>,
) {
    for (entity, needs, pop_trait) in query.iter() {
        if needs.stress >= 100.0 {
            let breakdown_type = match pop_trait {
                Some(Trait::Volatile) => BreakdownType::Pyromania,
                Some(Trait::Sluggish) => BreakdownType::Catatonia,
                _ => BreakdownType::Vandalism,
            };

            commands.entity(entity).insert(StressBreakdown {
                breakdown_type,
                duration: 500, // Arbitrary tick duration
            });
        }
    }
}

pub fn apply_breakdown_effects_system(
    mut commands: Commands,
    query: Query<Entity, (With<Pop>, With<StressBreakdown>, With<ActiveJob>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<ActiveJob>();
    }
}

pub fn tick_breakdown_duration_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StressBreakdown), With<Pop>>,
) {
    for (entity, mut breakdown) in query.iter_mut() {
        if breakdown.duration == 0 {
            commands.entity(entity).remove::<StressBreakdown>();
        } else {
            breakdown.duration -= 1;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action Overrides:** Instead of just removing `ActiveJob`, a breakdown should force the Pop's utility AI to prioritize destructive actions (e.g., Pyromania schedules an "Ignite Fire" action).
- **Stress Relief:** The breakdown should slowly drain `Stress` so the Pop naturally recovers over time.
- **Configurable Threshold:** Extract `100.0` and `500` ticks into global `const` values or a Settings resource.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A Pop with `Stress >= 100.0` gains the `StressBreakdown` component.
- [ ] The `BreakdownType` is correctly selected based on the Pop's `Trait`.
- [ ] The breakdown cancels their current `ActiveJob`.
- [ ] The breakdown duration decrements every tick, and the component is removed at 0.

## 7. Technical Guidance
- Add these systems to `Layer1SystemSet::Execution`.
- This feature works closely with `utility_ai`. If Utility AI is active, `StressBreakdown` could be implemented as an overriding Urge rather than a direct job cancellation, though a component flag is the simplest start.

## 8. Questions
*Builder: add questions here if spec is unclear.*
