//! Logic for the "Haul" action.
//!
//! # The Haul Action
//!
//! A clean colony is an efficient colony. Hauling serves two purposes:
//! 1.  **Sanitation**: Clearing waste and debris to improve beauty.
//! 2.  **Logistics**: Moving resources to stockpiles where they can be used for crafting.
//!
//! ## Logic Flow
//!
//! 1.  **Global Check**: Is there *any* stockpile space? If not, abort immediately (Optimization).
//! 2.  **Drop-off**: If the Pop is already carrying something, find the nearest valid stockpile.
//! 3.  **Pick-up**: If empty-handed, find the nearest loose item on the ground that:
//!     *   Has storage space available (e.g., won't pick up Wood if Wood storage is full).
//!     *   Is reachable.
//!
//! ## Capacity Management
//!
//! Haulers check [`ColonyResources`](crate::layer1::resources::ColonyResources) capacity before picking up an item.
//! This prevents "juggle hauling" where items are picked up and immediately dropped because there's nowhere to put them.

use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ItemProxy, PositionProxy};
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::calculate_context_score;
use bevy_ecs::prelude::*;

/// Evaluates the utility of hauling loose items to a [`crate::layer1::stockpile::Stockpile`].
///
/// **Returns:**
/// *   `Some((0.9, stockpile_entity))` if carrying an item (High priority to finish).
/// *   `Some((utility, item_entity))` if finding an item (Based on distance).
/// *   `None` if no stockpiles exist or no valid items found.
///
/// # Examples
///
/// ```rust,ignore
/// use scale::layer1::actions::haul::evaluate_haul;
/// use scale::layer1::map::GridPosition;
/// use scale::layer1::resources::{ColonyResources, ResourceType, Carrying};
/// use scale::layer1::utility_types::UtilityWeights;
/// use scale::layer1::utility_eval_types::{ItemProxy, PositionProxy};
/// use bevy_ecs::prelude::*;
///
/// let pop_pos = GridPosition { x: 0, y: 0 };
/// let weights = UtilityWeights::default();
/// let resources = ColonyResources::default();
///
/// // Scenario: Carrying Wood, need to find stockpile
/// let carrying = Some(Carrying {
///     resource_type: ResourceType::Wood,
///     amount: 1.0,
/// });
///
/// let stockpiles = vec![PositionProxy {
///     entity: Entity::PLACEHOLDER,
///     pos: GridPosition { x: 10, y: 0 },
/// }];
///
/// let result = evaluate_haul(
///     pop_pos,
///     &weights,
///     &[], // No items on ground needed for drop-off logic
///     &stockpiles,
///     &resources,
///     carrying
/// );
///
/// if let Some((utility, entity)) = result {
///     println!("Drop-off utility: {}", utility);
///     assert_eq!(utility, 0.9); // High priority to drop off
/// }
/// ```
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

        let utility = base_utility * context;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, item.entity));
        }
    }

    best
}
