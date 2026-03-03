use bevy_ecs::prelude::*;
use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;

/// Evaluates if a pop should fetch a tool.
#[must_use]
pub(crate) fn evaluate_fetch_tool(
    pop_pos: GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If already has a tool, no need to fetch
    if equipment.tool.is_some() {
        return None;
    }

    // If no tools available in colony, can't fetch
    if resources.tools < 1.0 {
        return None;
    }

    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, 0.9)
}
