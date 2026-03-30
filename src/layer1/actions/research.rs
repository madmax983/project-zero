use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing scientific research.
#[must_use]
pub(crate) fn evaluate_research(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    libraries: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If knowledge is full, no utility
    if resources.knowledge >= resources.max_knowledge {
        return None;
    }

    evaluate_candidates(pop_pos, weights, libraries, 0.4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_research() {
        let mut world = World::new();
        let target_entity = world.spawn_empty().id();

        let pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let mut resources = ColonyResources::default();
        resources.knowledge = 100.0;
        resources.max_knowledge = 100.0;

        let candidates = vec![ScorableCandidate {
            entity: target_entity,
            pos: GridPosition { x: 5, y: 5 },
            capacity: 10,
            usage: 0,
            score_bonus: 1.0,
            item_type: None,
            resource_type: None,
        }];

        // Test with max knowledge
        let result = evaluate_research(pos, &weights, &resources, &candidates);
        assert!(result.is_none());

        // Test with space for knowledge
        resources.knowledge = 50.0;
        let result_space = evaluate_research(pos, &weights, &resources, &candidates);
        assert!(result_space.is_some());
        let (score, target) = result_space.unwrap();
        assert!(score > 0.0);
        assert_eq!(target, target_entity);
    }
}
