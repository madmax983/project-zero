use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::refining::get_refining_recipe;
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use crate::layer1::utility_ai::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of refining resources at a building.
///
/// Checks if the colony has resources to refine (e.g., Wood -> Planks) and if
/// a building (e.g., Lumber Mill) is available.
///
/// # Returns
///
/// `Some((utility, target_entity))` if a valid job is found, `None` otherwise.
pub fn evaluate_refine<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    buildings: impl Iterator<Item = (Entity, &'a GridPosition, &'a Building, &'a RefiningProgress)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    for (entity, pos, building, progress) in buildings {
        // Check recipe validity
        let (can_afford, _, _) = get_refining_recipe(building.building_type, resources);

        if !can_afford {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity assumption (1 worker per mill for now)
            0, // Occupied assumption (handled by execution system or race condition accepted for MVP)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Refine, weights);

        // Boost utility if progress is already made
        let progress_bonus = if progress.current > 0.0 { 0.1 } else { 0.0 };

        let utility = (base_utility + progress_bonus) * context * success;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, entity));
        }
    }
    best
}
