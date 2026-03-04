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
