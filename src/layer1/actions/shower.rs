use crate::layer1::hygiene::SHOWER_WATER_COST;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::{UtilityWeights, need_response_curve};
use bevy_ecs::prelude::*;

/// Evaluates the utility of using a shower.
///
/// Driven by the Hygiene need.
#[must_use]
pub fn evaluate_shower(
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

    let result = evaluate_candidates(pop_pos, weights, candidates, urgency);
    if result.is_none() {
        // println!("Evaluate Shower: No candidates. Candidates len: {}", candidates.len());
    } else {
        // println!("Evaluate Shower: Found candidate with utility {:?}", result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_shower_score() {
        let pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hygiene: 0.1,
            ..Default::default()
        };
        let weights = UtilityWeights::default();
        let resources = ColonyResources {
            water: 10.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let result = evaluate_shower(pos, &needs, &weights, &resources, &candidates);

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score > 0.8, "Score {} should be high for dirty pop", score);
    }

    #[test]
    fn test_evaluate_shower_no_water() {
        let pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hygiene: 0.1,
            ..Default::default()
        };
        let weights = UtilityWeights::default();
        let resources = ColonyResources {
            water: 0.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let result = evaluate_shower(pos, &needs, &weights, &resources, &candidates);

        assert!(result.is_none(), "Should return None if no water");
    }
}
