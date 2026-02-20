use crate::layer1::items::{Clothing, ClothingType, Equipment, Item};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch clothing.
#[must_use]
pub(crate) fn evaluate_fetch_clothing(
    pop_pos: GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If already has clothing, no need to fetch
    if equipment.body.is_some() {
        return None;
    }

    // If no clothing available in colony, can't fetch
    if resources.clothing < 1.0 {
        return None;
    }

    // Use evaluate_candidates with default weights (implicitly handled by calculate_context_score inside)
    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, 0.95)
}

/// Executes the fetch clothing action.
pub fn handle_fetch_clothing(
    commands: &mut Commands,
    resources: &mut ColonyResources,
    pop_entity: Entity,
    equipment_opt: &mut Option<Mut<Equipment>>,
) {
    if resources.clothing >= 1.0 {
        resources.clothing -= 1.0;

        let clothing_entity = commands
            .spawn((
                Item::default(),
                Clothing {
                    clothing_type: ClothingType::Tunic, // Generic for now
                    insulation: 1.0,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        if let Some(eq) = equipment_opt {
            eq.body = Some(clothing_entity);
        } else {
            commands.entity(pop_entity).insert(Equipment {
                body: Some(clothing_entity),
                ..Default::default()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::utility_eval_types::ScorableCandidate;

    #[test]
    fn test_evaluate_fetch_clothing_needs_clothing() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment::default(); // No body
        let resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };
        // Fake entity for stockpile
        let stockpile_entity = Entity::from_raw(1);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 5, y: 0 },
        )];

        let result = evaluate_fetch_clothing(pop_pos, &equipment, &resources, &stockpiles);
        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, stockpile_entity);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_fetch_clothing_has_clothing() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment {
            body: Some(Entity::from_raw(2)),
            ..Default::default()
        };
        let resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };
        let stockpiles = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 5, y: 0 },
        )];

        let result = evaluate_fetch_clothing(pop_pos, &equipment, &resources, &stockpiles);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_fetch_clothing_no_resources() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment::default();
        let resources = ColonyResources {
            clothing: 0.0,
            ..Default::default()
        };
        let stockpiles = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 5, y: 0 },
        )];

        let result = evaluate_fetch_clothing(pop_pos, &equipment, &resources, &stockpiles);
        assert!(result.is_none());
    }
}
