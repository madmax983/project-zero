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
    current_insulation: f32,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
    temperature_grid: Option<&crate::layer1::temperature::TemperatureGrid>,
) -> Option<(f32, Entity)> {
    let mut score = 0.0;

    if current_insulation == 0.0 {
        score = 0.95; // Need clothes!
    } else if let Some(grid) = temperature_grid {
        // Check if freezing despite clothes
        let temp = grid.get(pop_pos.x as usize, pop_pos.y as usize);
        let safe_temp = 10.0 - (current_insulation * 30.0);
        if temp < safe_temp {
            // Check if we are already maxed out (e.g. Parka 2.0)?
            // If insulation is already high (e.g. >= 2.0), fetching won't help unless we have super-parka.
            // But currently max is Parka (2.0).
            // So if insulation < 2.0, try to fetch.
            if current_insulation < 2.0 {
                score = 0.99; // Upgrade needed immediately!
            }
        }
    }

    if score == 0.0 {
        return None;
    }

    // If no clothing available in colony, can't fetch
    if resources.clothing < 1.0 {
        return None;
    }

    // Use evaluate_candidates with default weights
    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, score)
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

        // Determine if upgrade is needed (simplistic logic: if already has item, give Parka)
        let mut is_upgrade = false;
        if let Some(eq) = equipment_opt {
            if eq.body.is_some() {
                is_upgrade = true;
                // Despawn old item? Or return to stockpile?
                // For MVP, despawn old item (discarded).
                if let Some(old_entity) = eq.body {
                    commands.entity(old_entity).despawn();
                }
            }
        }

        let (clothing_type, insulation) = if is_upgrade {
            (ClothingType::Parka, 2.0)
        } else {
            (ClothingType::Tunic, 1.0)
        };

        let clothing_entity = commands
            .spawn((
                Item::default(),
                Clothing {
                    clothing_type,
                    insulation,
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
        let current_insulation = 0.0;
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

        let result =
            evaluate_fetch_clothing(pop_pos, current_insulation, &resources, &stockpiles, None);
        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, stockpile_entity);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_fetch_clothing_has_clothing() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let current_insulation = 1.0;
        let resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };
        let stockpiles = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 5, y: 0 },
        )];

        // With no temperature grid, it should assume safe
        let result =
            evaluate_fetch_clothing(pop_pos, current_insulation, &resources, &stockpiles, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_fetch_clothing_no_resources() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let current_insulation = 0.0;
        let resources = ColonyResources {
            clothing: 0.0,
            ..Default::default()
        };
        let stockpiles = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 5, y: 0 },
        )];

        let result =
            evaluate_fetch_clothing(pop_pos, current_insulation, &resources, &stockpiles, None);
        assert!(result.is_none());
    }
}
