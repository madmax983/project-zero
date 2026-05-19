use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch a tool.
#[must_use]
pub(crate) fn evaluate_fetch_tool(
    pop_pos: GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If already has a tool, no need to fetch
    if equipment.tool.is_some() {
        return None;
    }

    // If no tools available in colony, can't fetch
    if resources.tools < 1.0 {
        return None;
    }

    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, 0.9)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::Equipment;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use bevy_ecs::prelude::Entity;

    #[test]
    fn test_evaluate_fetch_tool() {
        let mut equipment = Equipment::default();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let mut resources = ColonyResources::default();
        let stockpiles = vec![
            ScorableCandidate {
                entity: Entity::from_raw(1),
                pos: GridPosition { x: 1, y: 1 },
                capacity: 10,
                usage: 0,
                score_bonus: 0.0,
                resource_type: None,
                item_type: None,
                is_advanced_tech: false,
            }
        ];

        // Has tool
        equipment.tool = Some(Entity::from_raw(99));
        resources.tools = 5.0;
        assert_eq!(evaluate_fetch_tool(pop_pos, &equipment, &resources, &stockpiles), None);

        // No tool, but no tools in colony
        equipment.tool = None;
        resources.tools = 0.0;
        assert_eq!(evaluate_fetch_tool(pop_pos, &equipment, &resources, &stockpiles), None);

        // No tool, tools available
        resources.tools = 5.0;
        let result = evaluate_fetch_tool(pop_pos, &equipment, &resources, &stockpiles);
        assert!(result.is_some());
        let (score, entity) = result.unwrap();
        assert!(score > 0.0);
        assert_eq!(entity, Entity::from_raw(1));
    }
}
