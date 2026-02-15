use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::WorkDesignationProxy;
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing designated work (Mining, Building, etc.).
///
/// This checks all active [`crate::layer1::designation::Designation`]s (like "Mine this rock") and calculates
/// a score based on distance and the Pop's work ethic.
///
/// **Note:** This function receives `WorkDesignationProxy` which excludes `Repair` designations,
/// as those are handled separately by [`crate::layer1::actions::repair::evaluate_repair`].
///
/// # Returns
/// A tuple `(utility, designation_entity)` if a suitable task is found.
#[must_use]
pub fn evaluate_work(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: &[WorkDesignationProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    // Base utility for working (could depend on traits later)
    let base_utility = 0.5;

    for des in designations {
        // Repair designations are pre-filtered out

        let context = calculate_context_score(
            *pop_pos,
            Some(des.pos),
            1, // Capacity 1 (one worker per tile usually)
            0, // Occupied 0 (simplified for now)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Work, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, des.entity));
        }
    }
    best
}
