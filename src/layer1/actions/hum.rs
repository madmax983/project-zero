use bevy_ecs::prelude::*;
use crate::layer1::traits::Trait;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::{calculate_context_score, ActionType};

/// Evaluates the desire to listen to "The Hum".
#[must_use]
pub(crate) fn evaluate_listen_to_hum(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check Trait (Early Exit)
    if !data
        .traits
        .as_ref()
        .is_some_and(|t| t.0.contains(&Trait::Sensitive))
    {
        return (ActionType::ListenToTheHum, 0.0, None);
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    for candidate in &buffer.hum_sources {
        let intensity = candidate.score_bonus;
        let desire = intensity * (1.0 + data.stress);

        let context_score = calculate_context_score(
            data.pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            &data.weights,
        );

        let total_score = desire * context_score;

        if total_score > best_score {
            best_score = total_score;
            best_target = Some(candidate.entity);
        }
    }

    (ActionType::ListenToTheHum, best_score, best_target)
}
