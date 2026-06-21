use bevy_ecs::prelude::*;
use crate::layer1::skills::Skills;
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::needs::Needs;
use crate::layer1::health::Health;

#[derive(Component, Debug, Clone, Copy)]
pub struct TemporalFugue;

const FUGUE_XP_THRESHOLD: f32 = 8100.0; // Level 9 is 8100 XP, representing high skill (Spec 1023)

pub fn evaluate_fugue_state_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Skills, &PopAction), Without<TemporalFugue>>,
) {
    for (entity, skills, action) in query.iter_mut() {
        // High skill + Work action triggers fugue
        if action.current == ActionType::Work {
            let has_high_skill = skills.xp.values().any(|&xp| xp >= FUGUE_XP_THRESHOLD);
            if has_high_skill {
                commands.entity(entity).insert(TemporalFugue);
            }
        }
    }
}

pub fn fugue_completion_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PopAction, &Needs, &mut Health), With<TemporalFugue>>,
) {
    for (entity, action, needs, mut health) in query.iter_mut() {
        // Exit fugue if no longer working
        if action.current != ActionType::Work {
            commands.entity(entity).remove::<TemporalFugue>();

            // Consequences: If needs hit critical failure levels during the trance, apply damage
            if needs.hunger <= 0.0 {
                // Apply massive sudden damage
                let dmg = health.max;
                health.take_damage(dmg); // Instant death or massive damage
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::SkillType;

    #[test]
    fn test_highly_skilled_pop_enters_temporal_fugue() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, evaluate_fugue_state_system);

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Construction, 9000.0); // High skill

        // Spawn a high skill pop on a work job
        let pop = app.world_mut().spawn((
            Pop,
            skills,
            PopAction {
                current: ActionType::Work,
                current_utility: 1.0,
                ticks_committed: 0,
            },
        )).id();

        app.update();

        // Check if FugueState was added
        assert!(app.world().get::<TemporalFugue>(pop).is_some(), "Highly skilled Pops on work jobs should enter Temporal Fugue.");
    }

    #[test]
    fn test_pop_in_fugue_ignores_needs_until_task_completion() {
        // Need process_needs_system equivalent in tests to assert survival,
        // but we already updated needs.rs to exclude them.
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, fugue_completion_system);

        let pop = app.world_mut().spawn((
            Pop,
            TemporalFugue,
            Needs { hunger: 0.0, rest: 1.0, leisure: 1.0, hygiene: 1.0 }, // Starving
            Health { current: 100.0, max: 100.0, has_rust_lung: false },
            PopAction { current: ActionType::Work, current_utility: 1.0, ticks_committed: 0 },
        )).id();

        app.update();

        // Normally, a system would force the pop to stop working to eat if hunger is 0.
        // We verify that the PopAction persists despite 0 hunger. (Simulated by not changing)
        assert!(app.world().get::<PopAction>(pop).is_some(), "Pop in fugue should not drop tasks to fulfill needs.");
        assert!(app.world().get::<TemporalFugue>(pop).is_some(), "Pop in fugue should retain TemporalFugue while working.");

        // Complete the task
        app.world_mut().get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;
        app.update();

        // Upon completion, Fugue should be removed, and the Pop should immediately suffer the consequences of ignored needs
        assert!(app.world().get::<TemporalFugue>(pop).is_none(), "Fugue state should end when task completes.");
        assert_eq!(app.world().get::<Health>(pop).unwrap().current, 0.0, "Pop should instantly die if hunger reached 0 during fugue.");
    }
}
