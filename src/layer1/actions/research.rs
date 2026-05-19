use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing scientific research.
#[must_use]
pub(crate) fn evaluate_research(
    is_nostalgic: bool,
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    libraries: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If knowledge is full, no utility
    if is_nostalgic || resources.knowledge >= resources.max_knowledge {
        return None;
    }

    evaluate_candidates(pop_pos, weights, libraries, 0.4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::Entity;

    #[test]
    fn test_evaluate_research_nostalgic_or_full_knowledge() {
        let mut resources = ColonyResources::default();
        resources.knowledge = 100.0;
        resources.max_knowledge = 100.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let libraries = vec![];

        // Nostalgic should return None
        let result = evaluate_research(true, pop_pos, &weights, &resources, &libraries);
        assert_eq!(result, None);

        // Full knowledge should return None even if not nostalgic
        let result2 = evaluate_research(false, pop_pos, &weights, &resources, &libraries);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_evaluate_research_valid() {
        let mut resources = ColonyResources::default();
        resources.knowledge = 0.0;
        resources.max_knowledge = 100.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let libraries = vec![
            ScorableCandidate {
                entity: Entity::from_raw(1),
                pos: GridPosition { x: 1, y: 1 },
                capacity: 1,
                usage: 0,
                score_bonus: 0.0,
                resource_type: None,
                item_type: None,
                is_advanced_tech: false,
            }
        ];

        let result = evaluate_research(false, pop_pos, &weights, &resources, &libraries);
        assert!(result.is_some());
        let (score, entity) = result.unwrap();
        assert!(score > 0.0);
        assert_eq!(entity, Entity::from_raw(1));
    }
}
