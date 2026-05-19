use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Generic evaluator for simple actions (work, repair, etc.)
#[must_use]
pub(crate) fn evaluate_simple_action(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    base_utility: f32,
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, candidates, base_utility)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::Entity;

    #[test]
    fn test_evaluate_simple_action() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let candidates = vec![
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

        let result = evaluate_simple_action(pop_pos, &weights, &candidates, 0.5);
        assert!(result.is_some());
        let (score, entity) = result.unwrap();
        assert!(score > 0.0);
        assert_eq!(entity, Entity::from_raw(1));

        let empty_candidates: Vec<ScorableCandidate> = vec![];
        let result_empty = evaluate_simple_action(pop_pos, &weights, &empty_candidates, 0.5);
        assert_eq!(result_empty, None);
    }
}
