use crate::layer1::map::GridPosition;
use crate::layer1::science::Anomaly;
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of exploring an [`Anomaly`].
///
/// Anomalies (ruins, mysterious plants) provide unique rewards or trigger events.
/// Exploration is a medium-priority task (0.55 utility) - slightly better than
/// regular work but less critical than hauling food or healing.
#[must_use]
pub fn evaluate_explore<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    anomalies: impl Iterator<Item = (Entity, &'a GridPosition, &'a Anomaly)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.55;

    for (entity, pos, _) in anomalies {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity (simplified)
            0, // Occupied (simplified)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Explore, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}
