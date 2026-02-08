use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing designated work (Mining, Building, etc.).
///
/// This checks all active [`Designation`]s (like "Mine this rock") and calculates
/// a score based on distance and the Pop's work ethic.
///
/// **Note:** This function explicitly filters OUT [`DesignationType::Repair`] tasks,
/// as those are handled separately by [`evaluate_repair`] to prioritize maintenance.
///
/// # Returns
/// A tuple `(utility, designation_entity)` if a suitable task is found.
#[must_use]
pub fn evaluate_work<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: impl Iterator<Item = (Entity, &'a GridPosition, &'a Designation)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    // Base utility for working (could depend on traits later)
    let base_utility = 0.5;

    for (entity, pos, des) in designations {
        // Skip Repair designations (handled by evaluate_repair)
        if des.designation_type == DesignationType::Repair {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity 1 (one worker per tile usually)
            0, // Occupied 0 (simplified for now)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Work, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}
