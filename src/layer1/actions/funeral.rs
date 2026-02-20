use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of burying corpses.
#[must_use]
pub(crate) fn evaluate_bury_corpse(
    pop_pos: GridPosition,
    corpses: &[ScorableCandidate],
    graves: &[ScorableCandidate],
    weights: &UtilityWeights,
) -> Option<(f32, Entity)> {
    // Check if any grave is available
    if graves.is_empty() {
        return None;
    }

    evaluate_candidates(pop_pos, weights, corpses, 0.8)
}
