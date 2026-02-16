use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::RefiningProxy;
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
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
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    buildings: &[RefiningProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    for building in buildings {
        // Pre-filtered for schedule and recipe affordability

        let context = calculate_context_score(
            *pop_pos,
            Some(building.pos),
            1, // Capacity assumption (1 worker per mill for now)
            0, // Occupied assumption (handled by execution system or race condition accepted for MVP)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Refine, weights);

        // Boost utility if progress is already made
        let progress_bonus = if building.progress_current > 0.0 {
            0.1
        } else {
            0.0
        };

        let utility = (base_utility + progress_bonus) * context * success;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, building.entity));
        }
    }
    best
}
