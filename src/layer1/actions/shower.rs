use crate::layer1::hygiene::SHOWER_WATER_COST;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::need_response_curve;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of using a shower.
#[must_use]
pub(crate) fn evaluate_shower(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    candidates: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // Check if we can afford a shower
    if resources.water < SHOWER_WATER_COST {
        return None;
    }

    let urgency = need_response_curve(needs.hygiene);

    // If hygiene is high, urgency is low.
    // If urgency is very low, don't bother scanning.
    if urgency < 0.1 {
        return None;
    }

    evaluate_candidates(pop_pos, weights, candidates, urgency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_shower() {
        let mut world = World::new();
        let target_entity = world.spawn_empty().id();

        let pos = GridPosition { x: 0, y: 0 };
        let mut needs = Needs::default();
        needs.hygiene = 0.5;
        let weights = UtilityWeights::default();

        let mut resources = ColonyResources::default();
        resources.water = SHOWER_WATER_COST - 1.0;

        let candidates = vec![ScorableCandidate {
            entity: target_entity,
            pos: GridPosition { x: 5, y: 5 },
            capacity: 10,
            usage: 0,
            score_bonus: 1.0,
            item_type: None,
            resource_type: None,
        }];

        // Test without enough water
        let result = evaluate_shower(pos, &needs, &weights, &resources, &candidates);
        assert!(result.is_none());

        // Test with enough water but high hygiene (low urgency)
        resources.water = SHOWER_WATER_COST + 10.0;
        needs.hygiene = 0.95; // very clean
        let result_high_hygiene = evaluate_shower(pos, &needs, &weights, &resources, &candidates);
        assert!(result_high_hygiene.is_none());

        // Test with enough water and low hygiene
        needs.hygiene = 0.1; // very dirty
        let result_dirty = evaluate_shower(pos, &needs, &weights, &resources, &candidates);
        assert!(result_dirty.is_some());
        let (score, target) = result_dirty.unwrap();
        assert!(score > 0.0);
        assert_eq!(target, target_entity);
    }
}
