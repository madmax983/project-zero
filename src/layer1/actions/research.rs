use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::tech::Library;
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing scientific research at a [`Library`].
///
/// Research generates knowledge points, which unlock new [`crate::layer1::tech::Tech`].
///
/// **Constraints:**
/// *   Returns `None` if the colony's knowledge storage ([`ColonyResources`]) is full.
/// *   Requires an available worker slot at a [`Library`].
#[must_use]
pub fn evaluate_research<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    libraries: impl Iterator<Item = (Entity, &'a GridPosition, &'a Library)>,
) -> Option<(f32, Entity)> {
    // If knowledge is full, no utility
    if resources.knowledge >= resources.max_knowledge {
        return None;
    }

    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.4;

    for (entity, pos, _) in libraries {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            5, // Assumed capacity
            0, // Assumed occupied (not tracked yet)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Research, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}
