use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of refining resources at a building.
///
/// Checks if the colony has resources to refine (e.g., Wood -> Planks) and if
/// a building (e.g., Lumber Mill) is available.
///
/// # Returns
///
/// `Some((utility, target_entity))` if a valid job is found, `None` otherwise.
#[must_use]
pub(crate) fn evaluate_refine(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    buildings: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, buildings, 0.5)
}
