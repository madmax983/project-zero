//! Logic for the "Repair" action.

#![allow(clippy::match_same_arms)]
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of repairing damaged structures.
#[must_use]
pub(crate) fn evaluate_repair(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    designations: &[ScorableCandidate],
    structures: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    let best_des = evaluate_candidates(pop_pos, weights, designations, 0.6);
    let best_struct = evaluate_candidates(pop_pos, weights, structures, 0.6);

    match (best_des, best_struct) {
        (Some((u1, e1)), Some((u2, e2))) => {
            if u1 >= u2 {
                Some((u1, e1))
            } else {
                Some((u2, e2))
            }
        }
        (Some(res), None) => Some(res),
        (None, Some(res)) => Some(res),
        (None, None) => None,
    }
}
