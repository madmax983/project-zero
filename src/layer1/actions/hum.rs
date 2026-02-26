use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use crate::layer1::utility_types::{ActionType, calculate_context_score};
use crate::layer1::traits::Trait;
use bevy_ecs::prelude::*;

/// Evaluates the desire to listen to "The Hum".
///
/// Only pops with the `Sensitive` trait can hear it.
/// Score is based on:
/// - Hum Source Intensity (stored in `score_bonus`)
/// - Stress (higher stress -> higher attraction)
/// - Distance
#[must_use]
pub fn evaluate_listen_to_hum(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check Trait (Early Exit)
    if !data.traits.as_ref().map_or(false, |t| t.has(Trait::Sensitive)) {
        return (ActionType::ListenToTheHum, 0.0, None);
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    for candidate in &buffer.hum_sources {
        let intensity = candidate.score_bonus;

        // Base Desire Formula:
        // Intensity (0.0-1.0+) + Stress (0.0-1.0) * 0.5 + Low Leisure Need (High Urgency)
        // Actually spec says: "Restores Leisure... but increases Stress".
        // So high leisure need (low value) should increase desire?
        // Spec says: "The Hum soothes me".

        // Let's use: Intensity * (1.0 + Stress)
        // If stressed (1.0), desire doubles.
        let desire = intensity * (1.0 + data.stress);

        let context_score = calculate_context_score(
            data.pos,
            Some(candidate.pos),
            candidate.capacity, // Should be infinite? Or limited?
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
