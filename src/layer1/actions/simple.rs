use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Generic evaluator for simple actions (work, repair, etc.)
#[must_use]
pub(crate) fn evaluate_simple_action(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    base_utility: f32,
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, candidates, base_utility)
}
