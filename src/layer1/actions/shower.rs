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
    fn should_return_none_when_insufficient_water() {
        let pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hygiene: 0.0,
            ..Default::default()
        };
        let weights = UtilityWeights::default();
        let resources = ColonyResources {
            water: 0.0, // Less than SHOWER_WATER_COST
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(Entity::from_raw(1), pos)];

        let result = evaluate_shower(pos, &needs, &weights, &resources, &candidates);

        assert!(
            result.is_none(),
            "Expected None when water is insufficient for a shower"
        );
    }

    #[test]
    fn should_return_none_when_hygiene_is_high() {
        let pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hygiene: 1.0, // High hygiene means low urgency
            ..Default::default()
        };
        let weights = UtilityWeights::default();
        let resources = ColonyResources {
            water: 10.0, // Plenty of water
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(Entity::from_raw(1), pos)];

        let result = evaluate_shower(pos, &needs, &weights, &resources, &candidates);

        assert!(
            result.is_none(),
            "Expected None when hygiene is high (low urgency)"
        );
    }

    #[test]
    fn should_evaluate_candidates_when_hygiene_is_low_and_water_sufficient() {
        let pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hygiene: 0.0, // Low hygiene means high urgency
            ..Default::default()
        };
        let weights = UtilityWeights::default();
        let resources = ColonyResources {
            water: 10.0, // Plenty of water
            ..Default::default()
        };

        let target_entity = Entity::from_raw(1);
        let candidates = vec![ScorableCandidate::new(target_entity, pos)];

        let result = evaluate_shower(pos, &needs, &weights, &resources, &candidates);

        assert!(
            result.is_some(),
            "Expected Some result when hygiene is low and water is sufficient"
        );
        let (_, entity) = result.expect("Result should be Some");
        assert_eq!(entity, target_entity);
    }
}
