use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_eval_types::CapacityProxy;
use crate::layer1::utility_types::UtilityWeights;
use crate::layer1::utility_types::calculate_context_score;
use bevy_ecs::prelude::*;

/// Evaluates the utility of socializing at a tavern.
///
/// Socializing restores `leisure` need.
///
/// # Returns
///
/// `Some((utility, target_entity))` if a tavern is found, `None` otherwise.
#[must_use]
pub(crate) fn evaluate_socialize(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    taverns: &[CapacityProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    // Urgency
    let urgency = 1.0 - needs.leisure;

    for tavern in taverns {
        let context = calculate_context_score(
            pop_pos,
            Some(tavern.pos),
            tavern.capacity,
            tavern.usage,
            weights,
        );

        let utility = (base_utility + urgency) * context;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, tavern.entity));
        }
    }
    best
}
