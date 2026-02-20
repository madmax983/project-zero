use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of exploring an [`crate::layer1::science::Anomaly`].
#[must_use]
pub(crate) fn evaluate_explore(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    anomalies: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, anomalies, 0.55)
}
