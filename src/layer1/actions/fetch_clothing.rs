use crate::layer1::items::{Clothing, ClothingType, Equipment, Item};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::utility_types::manhattan_distance;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch clothing.
#[must_use]
pub(crate) fn evaluate_fetch_clothing(
    pop_pos: GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: &[PositionProxy],
) -> Option<(f32, Entity)> {
    // If already has clothing, no need to fetch
    if equipment.body.is_some() {
        return None;
    }

    // If no clothing available in colony, can't fetch
    if resources.clothing < 1.0 {
        return None;
    }

    // Find nearest stockpile
    let mut best_target = None;
    let mut min_dist = i32::MAX;

    for stockpile in stockpiles {
        let dist = manhattan_distance(&pop_pos, &stockpile.pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(stockpile.entity);
        }
    }

    best_target.map(|target| {
        // High urgency if naked (prevent hypothermia).
        // Hypothermia is dangerous, so this should be high priority.
        // Let's say 0.95.

        #[allow(clippy::cast_precision_loss)]
        let distance_factor = 1.0 / (min_dist as f32).mul_add(0.1, 1.0);
        let utility = 0.95 * distance_factor;

        (utility, target)
    })
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
    use crate::layer1::utility_eval_types::PositionProxy;

    #[test]
    fn test_evaluate_fetch_clothing_needs_clothing() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment::default(); // No body
        let resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };
        // Fake entity for stockpile
        // Note: Entity::from_raw(0) might be unsafe if world doesn't exist, but here we just store it in struct.
        // Bevy Entities are u64.
        let stockpile_entity = Entity::from_raw(1);
        let stockpiles = vec![PositionProxy {
            entity: stockpile_entity,
            pos: GridPosition { x: 5, y: 0 },
        }];

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
        let stockpiles = vec![PositionProxy {
            entity: Entity::from_raw(1),
            pos: GridPosition { x: 5, y: 0 },
        }];

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
        let stockpiles = vec![PositionProxy {
            entity: Entity::from_raw(1),
            pos: GridPosition { x: 5, y: 0 },
        }];

        let result = evaluate_fetch_clothing(pop_pos, &equipment, &resources, &stockpiles);
        assert!(result.is_none());
    }
}
