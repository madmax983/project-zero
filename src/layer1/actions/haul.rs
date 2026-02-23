//! Logic for the "Haul" action.

use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::{UtilityWeights, calculate_context_score};
use bevy_ecs::prelude::*;

/// Evaluates the utility of hauling loose items to a [`crate::layer1::stockpile::Stockpile`].
#[must_use]
#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_haul(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    items: &[ScorableCandidate],
    item_entities: &[ScorableCandidate],
    stockpiles: &[ScorableCandidate],
    resources: &ColonyResources,
    carrying: Option<crate::layer1::resources::Carrying>,
    carrying_item: Option<Entity>,
) -> Option<(f32, Entity)> {
    // 1. Check if any stockpile exists
    if stockpiles.is_empty() {
        return None;
    }

    // 2. If already carrying, go to stockpile (high priority)
    if carrying.is_some() || carrying_item.is_some() {
        return evaluate_candidates(pop_pos, weights, stockpiles, 0.9);
    }

    // 3. Find best item to pick up
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6;

    // Helper closure to update best
    let mut consider = |candidate: &ScorableCandidate| {
        let context = calculate_context_score(
            pop_pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            weights,
        );
        let utility = base_utility * context;
        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, candidate.entity));
        }
    };

    // Check Resource Items (with capacity check)
    for item in items {
        if let Some(res_type) = item.resource_type {
            let has_room = match res_type {
                crate::layer1::resources::ResourceType::Food => resources.food < resources.max_food,
                crate::layer1::resources::ResourceType::Wood => resources.wood < resources.max_wood,
                crate::layer1::resources::ResourceType::Stone => {
                    resources.stone < resources.max_stone
                }
                crate::layer1::resources::ResourceType::Ore => resources.ore < resources.max_ore,
                crate::layer1::resources::ResourceType::Metal => {
                    resources.metal < resources.max_metal
                }
                crate::layer1::resources::ResourceType::Planks => {
                    resources.planks < resources.max_planks
                }
                crate::layer1::resources::ResourceType::Blocks => {
                    resources.blocks < resources.max_blocks
                }
                crate::layer1::resources::ResourceType::Waste => {
                    resources.waste < resources.max_waste
                }
                crate::layer1::resources::ResourceType::Rations => {
                    resources.rations < resources.max_rations
                }
                crate::layer1::resources::ResourceType::Fuel => resources.fuel < resources.max_fuel,
                crate::layer1::resources::ResourceType::Alcohol => {
                    resources.alcohol < resources.max_alcohol
                }
            };

            if !has_room {
                continue;
            }
        }
        consider(item);
    }

    // Check Generic Item Entities
    for item in item_entities {
        consider(item);
    }

    best
}
