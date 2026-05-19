use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::{calculate_context_score, UtilityWeights};
use bevy_ecs::prelude::*;

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
#[allow(clippy::too_many_arguments)]
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
                crate::layer1::resources::ResourceType::MemoryCore => {
                    resources.memory_cores < resources.max_memory_cores
                }
                crate::layer1::resources::ResourceType::VoidAle
                | crate::layer1::resources::ResourceType::HyperValuable => {
                    resources.memory_cores < resources.max_memory_cores
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::ItemType;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{Carrying, ColonyResources, ResourceType};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::Entity;

    fn base_resources() -> ColonyResources {
        ColonyResources {
            max_food: 100.0,
            food: 50.0,
            max_wood: 100.0,
            wood: 50.0,
            ..Default::default()
        }
    }

    fn test_entity(id: u64) -> Entity {
        Entity::from_raw(id as u32)
    }

    #[test]
    fn should_target_gene_bank_when_carrying_genetic_sample() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            availability_weight: 1.0,
        };
        let gene_bank_entity = test_entity(1);
        let gene_banks = vec![ScorableCandidate::new(
            gene_bank_entity,
            GridPosition { x: 1, y: 1 },
        )];
        let resources = base_resources();
        let carrying_item = Some(test_entity(2));
        let carrying_item_type = Some(ItemType::GeneticSample);

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &[],
            &[],
            &[],
            &gene_banks,
            &resources,
            None,
            carrying_item,
            carrying_item_type,
        );

        assert!(result.is_some(), "Should return a gene bank target");
        let (utility, target) = result.unwrap();
        assert_eq!(target, gene_bank_entity);
        assert!(utility > 0.0);
    }

    #[test]
    fn should_return_none_if_carrying_but_no_stockpile_exists() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            availability_weight: 1.0,
        };
        let resources = base_resources();
        let carrying = Some(Carrying {
            amount: 10.0,
            resource_type: ResourceType::Food,
        });

        // Empty stockpiles
        let stockpiles = vec![];

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &[],
            &[],
            &stockpiles,
            &[],
            &resources,
            carrying,
            None,
            None,
        );

        assert!(result.is_none(), "Should fail if nowhere to drop off");
    }

    #[test]
    fn should_target_stockpile_if_carrying_generic_resource() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            availability_weight: 1.0,
        };
        let stockpile_entity = test_entity(3);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 1, y: 1 },
        )];
        let resources = base_resources();
        let carrying = Some(Carrying {
            amount: 10.0,
            resource_type: ResourceType::Food,
        });

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &[],
            &[],
            &stockpiles,
            &[],
            &resources,
            carrying,
            None,
            None,
        );

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, stockpile_entity);
        assert!(utility > 0.0);
    }

    #[test]
    fn should_pick_up_item_if_empty_handed_and_stockpiles_exist() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            availability_weight: 1.0,
        };
        let stockpile_entity = test_entity(3);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 1, y: 1 },
        )];
        let resources = base_resources();

        let item_entity = test_entity(4);
        let mut item_cand = ScorableCandidate::new(item_entity, GridPosition { x: 2, y: 2 });
        item_cand.resource_type = Some(ResourceType::Food);
        let items = vec![item_cand];

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &items,
            &[],
            &stockpiles,
            &[],
            &resources,
            None,
            None,
            None,
        );

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, item_entity);
        assert!(utility > 0.0);
    }

    #[test]
    fn should_ignore_pickup_if_resource_at_max_capacity() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            availability_weight: 1.0,
        };
        let stockpile_entity = test_entity(3);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 1, y: 1 },
        )];

        let mut resources = base_resources();
        // Set food to max capacity
        resources.food = resources.max_food;

        let item_entity = test_entity(4);
        let mut item_cand = ScorableCandidate::new(item_entity, GridPosition { x: 2, y: 2 });
        item_cand.resource_type = Some(ResourceType::Food);
        let items = vec![item_cand];

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &items,
            &[],
            &stockpiles,
            &[],
            &resources,
            None,
            None,
            None,
        );

        assert!(
            result.is_none(),
            "Should ignore pickup if we have no room for it in the colony"
        );
    }

    #[test]
    fn should_pick_up_generic_item_entity() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let stockpile_entity = test_entity(3);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 1, y: 1 },
        )];
        let resources = base_resources();

        let item_entity = test_entity(5);
        let mut item_cand = ScorableCandidate::new(item_entity, GridPosition { x: 2, y: 2 });
        item_cand.item_type = Some(ItemType::Tool);
        let item_entities = vec![item_cand];

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &[],
            &item_entities,
            &stockpiles,
            &[],
            &resources,
            None,
            None,
            None,
        );

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, item_entity);
        assert!(utility > 0.0);
    }
}
