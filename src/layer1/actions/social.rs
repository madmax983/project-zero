use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of socializing at a tavern.
#[must_use]
pub(crate) fn evaluate_socialize(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    taverns: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // Urgency based on leisure need
    let urgency = (1.0 - needs.leisure) * 1.5;
    evaluate_candidates(pop_pos, weights, taverns, urgency)
}
