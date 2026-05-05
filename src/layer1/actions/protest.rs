use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_eval_types::ScorableCandidate;
use crate::layer1::mind::utility_types::UtilityWeights;

/// Evaluates the utility of joining a protest mob.
#[must_use]
pub fn evaluate_protest(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    mobs: &[ScorableCandidate],
) -> Option<(f32, bevy_ecs::prelude::Entity)> {
    let mut best_score = -1.0;
    let mut best_target = None;

    for mob in mobs {
        // High base utility for protesting when striking
        let base_utility = 0.8;

        // Calculate distance penalty using Manhattan distance
        let dist = crate::layer1::mind::utility_ai::manhattan_distance(&pop_pos, &mob.pos) as f32;
        let distance_penalty = (dist / 100.0) * weights.distance_weight;

        let score = (base_utility - distance_penalty).max(0.0);

        if score > best_score {
            best_score = score;
            best_target = Some(mob.entity);
        }
    }

    if best_score > 0.0 {
        best_target.map(|t| (best_score, t))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::Entity;

    #[test]
    fn test_evaluate_protest_empty_mobs() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            ..Default::default()
        };
        let mobs: Vec<ScorableCandidate> = vec![];

        let result = evaluate_protest(pop_pos, &weights, &mobs);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_protest_selects_best_mob() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 1.0,
            ..Default::default()
        };

        let mob1 = Entity::from_raw(1);
        let mob2 = Entity::from_raw(2);

        let cand1 = ScorableCandidate::new(mob1, GridPosition { x: 10, y: 0 }); // Distance 10 (penalty 0.1) -> Score 0.7
        let cand2 = ScorableCandidate::new(mob2, GridPosition { x: 50, y: 0 }); // Distance 50 (penalty 0.5) -> Score 0.3

        let mobs = vec![cand1, cand2];

        let result = evaluate_protest(pop_pos, &weights, &mobs);
        assert!(result.is_some());
        let (score, entity) = result.unwrap();
        assert_eq!(entity, mob1);
        assert!((score - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_protest_too_far() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 10.0,
            ..Default::default()
        }; // High penalty

        let mob1 = Entity::from_raw(1);
        let cand1 = ScorableCandidate::new(mob1, GridPosition { x: 100, y: 0 }); // Distance 100 -> Penalty 10.0 -> Score 0.0 (or negative)
        let mobs = vec![cand1];

        let result = evaluate_protest(pop_pos, &weights, &mobs);
        assert!(result.is_none());
    }
}
