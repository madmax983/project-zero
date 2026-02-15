use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::utility_types::{UtilityWeights, manhattan_distance};
use bevy_ecs::prelude::*;

/// Evaluates the utility of burying corpses.
///
/// Returns `Some((utility, corpse_entity))` if viable.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn evaluate_bury_corpse(
    pop_pos: &GridPosition,
    corpses: &[PositionProxy],
    graves: &[PositionProxy],
    weights: &UtilityWeights,
) -> Option<(f32, Entity)> {
    // Check if any grave is available (pre-filtered in buffer)
    if graves.is_empty() {
        return None;
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    for corpse in corpses {
        let dist = manhattan_distance(pop_pos, &corpse.pos);

        // Urgency: 0.8 base (high priority to clean up)
        // Distance penalty
        // Weight influence

        // mul_add usage: (dist as f32).mul_add(0.1, 1.0)
        let distance_factor = 1.0 / (dist as f32).mul_add(0.1, 1.0);

        // Using `distance_weight` from weights
        let score = 0.8 * distance_factor.powf(weights.distance_weight);

        if score > best_score {
            best_score = score;
            best_target = Some(corpse.entity);
        }
    }

    best_target.map(|t| (best_score, t))
}
