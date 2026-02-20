use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing scientific research at a [`crate::layer1::tech::Library`].
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
