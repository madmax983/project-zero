use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

#[allow(dead_code)]
pub fn evaluate_sabotage(
    _pop_pos: GridPosition,
    _weights: &UtilityWeights,
    targets: &[crate::layer1::mind::utility_eval_types::ScorableCandidate],
) -> Option<(f32, Entity)> {
    if targets.is_empty() {
        return None;
    }

    let base_weight = 0.8; // Sabotage has high weight for cult members

    // We just take the first target for simplicity in Utility AI right now.
    // Real implementation would calculate distances.
    let best_target = &targets[0];

    // A simplified scoring logic, as target context might be complex
    let score = base_weight;

    if score > 0.0 {
        Some((score, best_target.entity))
    } else {
        None
    }
}
