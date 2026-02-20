//! Logic for the "Work" action (Mining, Building, etc.).

use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing designated work.
#[must_use]
pub(crate) fn evaluate_work(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    designations: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, designations, 0.5)
}
