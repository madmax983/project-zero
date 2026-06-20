use bevy_ecs::prelude::*;

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::pop::Pop;
use crate::layer1::skills::Skills;
use crate::layer1::utility_types::{ActionType, PopAction};

/// Allows highly skilled Pops to enter a specialized trance where they work at massive speeds
/// but completely ignore all physiological needs.
#[derive(Component, Debug, Clone)]
pub struct TemporalFugue;

/// Threshold for a skill to be considered high enough to trigger Temporal Fugue.
/// The spec says 90. In our `Skills::get_level` system, level = sqrt(xp/100).
/// So level 90 requires 90^2 * 100 = 810,000 XP.
/// Or maybe it means level 90? The spec just says `u32 = 90`. We will check if level >= 90.
pub const FUGUE_SKILL_THRESHOLD: u32 = 90;

#[allow(clippy::type_complexity)]
pub fn evaluate_fugue_state_system(
    mut commands: Commands,
    query: Query<(Entity, &Skills, &PopAction), (With<Pop>, Without<TemporalFugue>)>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for (entity, skills, action) in query.iter() {
        if action.current == ActionType::Work {
            let has_high_skill = skills
                .xp
                .keys()
                .any(|&skill_type| skills.get_level(skill_type) >= FUGUE_SKILL_THRESHOLD);
            if has_high_skill {
                commands.entity(entity).insert(TemporalFugue);
                chronicle.send(AddChronicleEvent {
                    text: "A colonist has entered a Temporal Fugue, working at blinding speeds but ignoring their own survival.".to_string(),
                    importance: EventImportance::Major,
                });
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn fugue_completion_system(
    mut commands: Commands,
    query: Query<(Entity, &PopAction), (With<Pop>, With<TemporalFugue>)>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for (entity, action) in query.iter() {
        if action.current != ActionType::Work {
            commands.entity(entity).remove::<TemporalFugue>();
            chronicle.send(AddChronicleEvent {
                text: "A colonist has snapped out of their Temporal Fugue, suddenly realizing their bodily needs.".to_string(),
                importance: EventImportance::Minor,
            });
        }
    }
}
