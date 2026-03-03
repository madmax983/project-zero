use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::{calculate_context_score, UtilityWeights};

/// Evaluates the utility of hauling loose items.
///
/// # Logistics Priority
///
/// Not all items are created equal. The AI prioritizes destinations:
///
/// 1.  **Gene Banks (Critical):** If carrying a `GeneticSample`, finding a Gene Bank
///     is Priority #1 (0.95 utility). We don't want rare DNA rotting in a pocket.
/// 2.  **Stockpiles (Standard):** If carrying anything else, find a Stockpile (0.90).
/// 3.  **Pickup (Idle):** If empty-handed, find the nearest loose item (0.60 base).
///     This is a lower priority than "real work" (Mining/Building).
#[must_use]
pub(crate) fn evaluate_haul(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    items: &[ScorableCandidate],
    item_entities: &[ScorableCandidate],
    stockpiles: &[ScorableCandidate],
    gene_banks: &[ScorableCandidate],
    resources: &ColonyResources,
    carrying: Option<crate::layer1::resources::Carrying>,
    carrying_item: Option<Entity>,
    carrying_item_type: Option<crate::layer1::items::ItemType>,
) -> Option<(f32, Entity)> {
    // 1. If carrying GeneticSample, target GeneBank
    if carrying_item.is_some()
        && matches!(
            carrying_item_type,
            Some(crate::layer1::items::ItemType::GeneticSample)
        )
        && !gene_banks.is_empty()
    {
        return evaluate_candidates(pop_pos, weights, gene_banks, 0.95);
    }
    // If no gene banks, might fall through or fail.
    // For now, let it fall through to stockpiles if any (though unlikely to accept it if filtering implemented)

    // 2. Check if any stockpile exists (Standard Hauling)
    if stockpiles.is_empty() {
        // If we are carrying something but no stockpile exists, and it wasn't a genetic sample for a gene bank,
        // we are stuck.
        return None;
    }

    // 3. If already carrying, go to stockpile (high priority)
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
                crate::layer1::resources::ResourceType::Scrap => {
                    resources.scrap < resources.max_scrap
                }
                crate::layer1::resources::ResourceType::Tools => {
                    resources.tools < resources.max_tools
                }
                crate::layer1::resources::ResourceType::BuildingPermit => {
                    resources.building_permits < resources.max_building_permits
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
