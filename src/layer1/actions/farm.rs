use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::CapacityProxy;
use crate::layer1::utility_types::UtilityWeights;
use crate::layer1::utility_types::calculate_context_score;
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
    farms: &[CapacityProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    for farm in farms {
        // Pre-filtered for schedule and capacity in evaluate_actions_system

        let context =
            calculate_context_score(pop_pos, Some(farm.pos), farm.capacity, farm.usage, weights);

        let utility = base_utility * context;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, farm.entity));
        }
    }
    best
}
