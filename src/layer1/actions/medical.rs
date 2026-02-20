use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of seeking medical care.
#[must_use]
pub(crate) fn evaluate_seek_medical_care(
    pop_pos: GridPosition,
    _needs: &crate::layer1::needs::Needs,
    health: Health,
    weights: &UtilityWeights,
    hospitals: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    if health.current >= health.max * 0.95 {
        return None;
    }

    let health_pct = health.current / health.max;
    let urgency = (1.0 - health_pct) * 2.0;

    evaluate_candidates(pop_pos, weights, hospitals, urgency)
}
