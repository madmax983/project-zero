use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ItemProxy, PositionProxy};
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;

/// Evaluates the utility of hauling loose items to a [`Stockpile`](crate::layer1::stockpile::Stockpile).
///
/// A clean colony is a happy colony. Hauling items prevents beauty decay
/// and makes resources available for crafting.
///
/// **Logic:**
/// 1.  Checks if *any* [`Stockpile`](crate::layer1::stockpile::Stockpile) exists (short-circuit optimization).
/// 2.  Iterates through all [`ItemProxy`] entities on the map.
/// 3.  Checks if the colony has storage capacity for that specific resource type.
///     (e.g., won't haul wood if `wood >= max_wood`).
/// 4.  Scores based on distance to the item.
///
/// **Returns:** `Some((utility, item_entity))`
#[must_use]
pub(crate) fn evaluate_haul(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    items: &[ItemProxy],
    stockpiles: &[PositionProxy],
    resources: &ColonyResources,
    carrying: Option<crate::layer1::resources::Carrying>,
) -> Option<(f32, Entity)> {
    // 1. Check if any stockpile exists (optimization: no point hauling if nowhere to put it)
    if stockpiles.is_empty() {
        return None;
    }

    // 2. If already carrying, go to stockpile
    if carrying.is_some() {
        // Find closest stockpile
        let mut best_stockpile = None;
        let mut min_dist = i32::MAX;

        for stockpile in stockpiles {
            let dist = crate::layer1::utility_types::manhattan_distance(&pop_pos, &stockpile.pos);
            if dist < min_dist {
                min_dist = dist;
                best_stockpile = Some(stockpile.entity);
            }
        }

        return best_stockpile.map(|entity| {
            // High utility to finish the job
            // We use a high base because completing a haul is efficient
            (0.9, entity)
        });
    }

    // 3. Find closest item we have room for
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Slightly higher than work (0.5) to keep map clean

    for item in items {
        // Check capacity
        let has_room = match item.resource_type {
            crate::layer1::resources::ResourceType::Food => resources.food < resources.max_food,
            crate::layer1::resources::ResourceType::Wood => resources.wood < resources.max_wood,
            crate::layer1::resources::ResourceType::Stone => resources.stone < resources.max_stone,
            crate::layer1::resources::ResourceType::Ore => resources.ore < resources.max_ore,
            crate::layer1::resources::ResourceType::Metal => resources.metal < resources.max_metal,
            crate::layer1::resources::ResourceType::Planks => {
                resources.planks < resources.max_planks
            }
            crate::layer1::resources::ResourceType::Blocks => {
                resources.blocks < resources.max_blocks
            }
            crate::layer1::resources::ResourceType::Waste => resources.waste < resources.max_waste,
            crate::layer1::resources::ResourceType::Rations => {
                resources.rations < resources.max_rations
            }
            crate::layer1::resources::ResourceType::Fuel => resources.fuel < resources.max_fuel,
        };

        if !has_room {
            continue;
        }

        let context = calculate_context_score(
            pop_pos,
            Some(item.pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let success = calculate_success_modifier(ActionType::Haul, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, item.entity));
        }
    }

    best
}
