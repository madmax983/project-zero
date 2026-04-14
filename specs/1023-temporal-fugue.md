# 1023: Temporal Fugue

## 1. Overview
Temporal Fugue (or "Flow State") allows highly skilled Pops to enter a specialized trance where they work at massive speeds (e.g., 300%) but completely ignore all physiological needs (Hunger, Sleep) until they collapse or complete the task. This introduces a risk/reward tension where a critical repair might be completed instantly, but the assigned Pop may die of starvation immediately after.

## 2. Dependencies
- Layer 1 `Pop` entity and skill levels.
- Layer 1 `Utility AI` (Work task execution speed).
- Layer 1 `Needs` system (Hunger, Sleep).
- Layer 1 `Medical`/`Health` (Collapse/Death from zero needs).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, SkillLevel};
    use crate::layer1::utility_ai::{WorkAction, WorkProgressEvent};
    use crate::layer1::needs::{Hunger, Sleep};
    use crate::layer1::health::{Health, DamageEvent};

    #[test]
    fn test_highly_skilled_pop_enters_temporal_fugue() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_fugue_state_system);

        // Spawn a high skill pop on an urgent job
        let pop = app.world_mut().spawn((
            Pop,
            SkillLevel { value: 95 }, // High skill
            WorkAction { is_urgent: true, progress: 0.0, speed_multiplier: 1.0 },
        )).id();

        app.update();

        // Check if FugueState was added and speed increased
        assert!(app.world().get::<TemporalFugue>(pop).is_some(), "Highly skilled Pops on urgent jobs should enter Temporal Fugue.");
        let work = app.world().get::<WorkAction>(pop).unwrap();
        assert_eq!(work.speed_multiplier, 3.0, "Fugue state should multiply work speed by 3x.");
    }

    #[test]
    fn test_pop_in_fugue_ignores_needs_until_task_completion() {
        let mut app = App::new();
        app.add_systems(Update, (process_needs_system, fugue_completion_system).chain());

        let pop = app.world_mut().spawn((
            Pop,
            TemporalFugue,
            Hunger { value: 0.0, decay_rate: 1.0 }, // Starving
            WorkAction { is_urgent: true, progress: 50.0, speed_multiplier: 3.0 },
        )).id();

        // Normally, a system would force the pop to stop working to eat if hunger is 0.
        // We verify that the WorkAction persists despite 0 hunger.
        app.update();
        assert!(app.world().get::<WorkAction>(pop).is_some(), "Pop in fugue should not drop tasks to fulfill needs.");

        // Complete the task
        app.world_mut().get_mut::<WorkAction>(pop).unwrap().progress = 100.0;
        app.update();

        // Upon completion, Fugue should be removed, and the Pop should immediately suffer the consequences of ignored needs
        assert!(app.world().get::<TemporalFugue>(pop).is_none(), "Fugue state should end when task completes.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/temporal_fugue.rs
use bevy::prelude::*;
use crate::layer1::pop::SkillLevel;
use crate::layer1::utility_ai::WorkAction;
use crate::layer1::needs::Hunger;

#[derive(Component)]
pub struct TemporalFugue;

const FUGUE_SKILL_THRESHOLD: u32 = 90;
const FUGUE_SPEED_MULTIPLIER: f32 = 3.0;

pub fn evaluate_fugue_state_system(
    mut commands: Commands,
    mut query: Query<(Entity, &SkillLevel, &mut WorkAction), Without<TemporalFugue>>,
) {
    for (entity, skill, mut work) in query.iter_mut() {
        // High skill + urgent task triggers fugue
        if skill.value >= FUGUE_SKILL_THRESHOLD && work.is_urgent {
            commands.entity(entity).insert(TemporalFugue);
            work.speed_multiplier = FUGUE_SPEED_MULTIPLIER;
        }
    }
}

pub fn fugue_completion_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WorkAction), With<TemporalFugue>>,
) {
    for (entity, mut work) in query.iter_mut() {
        if work.progress >= 100.0 {
            // Task complete, exit fugue
            commands.entity(entity).remove::<TemporalFugue>();
            work.speed_multiplier = 1.0; // Reset
            // (The utility AI will naturally despawn the WorkAction next tick)
        }
    }
}

// Stub for the needs system to demonstrate ignoring needs
pub fn process_needs_system(
    query: Query<(Entity, &Hunger, Option<&TemporalFugue>)>,
) {
    for (_entity, hunger, fugue) in query.iter() {
        if hunger.value <= 0.0 && fugue.is_none() {
            // Normal behavior: interrupt task to eat
        } else if hunger.value <= 0.0 && fugue.is_some() {
            // Fugue behavior: keep working, ignore the starvation (until completion)
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Consequences:** The system needs a hook to apply massive sudden damage or instant death the moment `TemporalFugue` is removed if the underlying needs (Hunger, Sleep) hit critical failure levels during the trance.
- **Player Control:** Is Fugue an automatic response, or an Edict/Policy the player activates? If automatic, players might get frustrated when their best engineers randomly die fixing a door. It should probably require a specific `Stance` or manual override.
- **Visual Feedback:** The UI needs to heavily flag a Pop in Fugue so the player understands why they are moving incredibly fast but their health bar is flashing red.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_highly_skilled_pop_enters_temporal_fugue` passes.
- [ ] Test `test_pop_in_fugue_ignores_needs_until_task_completion` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Integration with the `Utility AI` is tricky. You'll likely need to modify the `evaluate_actions_system` to heavily penalize or disable the "Eat" and "Sleep" actions specifically if the `TemporalFugue` component is present.

## 8. Questions
*Builder: add questions here if spec is unclear.*
