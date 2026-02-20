use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of farming.
///
/// Checks for farms with available capacity.
///
/// # Returns
///
/// `Some((utility, target_entity))` if a valid farm is found, `None` otherwise.
#[must_use]
pub(crate) fn evaluate_farm(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    farms: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, farms, 0.5)
}
