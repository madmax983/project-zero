use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceItem};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of hauling loose items to a [`Stockpile`].
///
/// A clean colony is a happy colony. Hauling items prevents beauty decay
/// and makes resources available for crafting.
///
/// **Logic:**
/// 1.  Checks if *any* [`Stockpile`] exists (short-circuit optimization).
/// 2.  Iterates through all [`ResourceItem`] entities on the map.
/// 3.  Checks if the colony has storage capacity for that specific resource type.
///     (e.g., won't haul wood if `wood >= max_wood`).
/// 4.  Scores based on distance to the item.
///
/// **Returns:** `Some((utility, item_entity))`
#[must_use]
pub fn evaluate_haul<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    items: impl Iterator<Item = (Entity, &'a GridPosition, &'a ResourceItem)>,
    stockpiles: impl Iterator<Item = (Entity, &'a GridPosition, &'a Stockpile)>,
    resources: &ColonyResources,
) -> Option<(f32, Entity)> {
    // 1. Check if any stockpile exists (optimization: no point hauling if nowhere to put it)
    if stockpiles.count() == 0 {
        return None;
    }

    // 2. Find closest item we have room for
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Slightly higher than work (0.5) to keep map clean

    for (entity, pos, item) in items {
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
        };

        if !has_room {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let success = calculate_success_modifier(ActionType::Haul, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }

    best
}
