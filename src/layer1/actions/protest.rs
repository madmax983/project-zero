use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_eval_types::ScorableCandidate;
use crate::layer1::mind::utility_types::UtilityWeights;

/// Evaluates the utility of joining a protest mob.
#[must_use]
pub fn evaluate_protest(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    mobs: &[ScorableCandidate],
) -> Option<(f32, bevy_ecs::prelude::Entity)> {
    let mut best_score = -1.0;
    let mut best_target = None;

    for mob in mobs {
        // High base utility for protesting when striking
        let base_utility = 0.8;

        // Calculate distance penalty using Manhattan distance
        let dist = crate::layer1::mind::utility_ai::manhattan_distance(&pop_pos, &mob.pos) as f32;
        let distance_penalty = (dist / 100.0) * weights.distance_weight;

        let score = (base_utility - distance_penalty).max(0.0);

        if score > best_score {
            best_score = score;
            best_target = Some(mob.entity);
        }
    }

    if best_score > 0.0 {
        best_target.map(|t| (best_score, t))
    } else {
        None
    }
}
