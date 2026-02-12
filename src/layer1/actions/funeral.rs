use crate::layer1::funeral::{Corpse, Grave};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::{UtilityWeights, manhattan_distance};
use bevy_ecs::prelude::*;

/// Evaluates the utility of burying corpses.
///
/// Returns `Some((utility, corpse_entity))` if viable.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_bury_corpse<'a>(
    pop_pos: &GridPosition,
    corpses_iter: impl Iterator<Item = (Entity, &'a GridPosition, &'a Corpse)>,
    graves_iter: impl Iterator<Item = &'a Grave>,
    weights: &UtilityWeights,
) -> Option<(f32, Entity)> {
    // Check if any grave is available
    let has_empty_grave = graves_iter.into_iter().any(|g| !g.occupied);

    if !has_empty_grave {
        return None;
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    for (entity, corpse_pos, _) in corpses_iter {
        let dist = manhattan_distance(pop_pos, corpse_pos);

        // Urgency: 0.8 base (high priority to clean up)
        // Distance penalty
        // Weight influence

        // mul_add usage: (dist as f32).mul_add(0.1, 1.0)
        let distance_factor = 1.0 / (dist as f32).mul_add(0.1, 1.0);

        // Using `distance_weight` from weights
        let score = 0.8 * distance_factor.powf(weights.distance_weight);

        if score > best_score {
            best_score = score;
            best_target = Some(entity);
        }
    }

    best_target.map(|t| (best_score, t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_bury_corpse_no_grave() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let pos = GridPosition { x: 1, y: 0 };
        let corpse = Corpse {
            name: "A".into(),
            decay: 0.0,
        };
        let corpses = vec![(Entity::from_raw(1), &pos, &corpse)];

        // Empty graves iter
        let graves: Vec<&Grave> = vec![];

        let result =
            evaluate_bury_corpse(&pop_pos, corpses.into_iter(), graves.into_iter(), &weights);

        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_bury_corpse_finds_best() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let pos1 = GridPosition { x: 1, y: 0 };
        let corpse1 = Corpse {
            name: "A".into(),
            decay: 0.0,
        };
        let c1 = (Entity::from_raw(1), &pos1, &corpse1);

        let pos2 = GridPosition { x: 10, y: 0 };
        let corpse2 = Corpse {
            name: "B".into(),
            decay: 0.0,
        };
        let c2 = (Entity::from_raw(2), &pos2, &corpse2);

        let corpses = vec![c1, c2];

        // One empty grave
        let grave = Grave {
            occupied: false,
            corpse_name: None,
        };
        let graves = vec![&grave];

        let result =
            evaluate_bury_corpse(&pop_pos, corpses.into_iter(), graves.into_iter(), &weights);

        assert!(result.is_some());
        let (score, entity) = result.unwrap();
        assert_eq!(entity, c1.0); // Should pick closer one
        assert!(score > 0.0);
    }
}
