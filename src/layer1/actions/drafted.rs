use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

/// Evaluates actions for a drafted pop (combat).
pub(crate) fn evaluate_drafted_behavior(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    data.drafted?;

    let mut best_action = ActionType::Idle;
    let mut best_utility = 0.9; // Just stand there ready
    let mut best_target = None;

    // Helper to evaluate fight
    let evaluate_fight = || -> Option<(f32, Entity)> {
        // Find nearest enemy
        let mut best_fight_target = None;
        let mut min_dist = f32::MAX;

        for candidate in &buffer.enemies {
            #[allow(clippy::cast_precision_loss)]
            let dist = data.pos.distance_chebyshev(candidate.pos) as f32;
            if dist < min_dist {
                min_dist = dist;
                best_fight_target = Some(candidate.entity);
            }
        }

        best_fight_target.map(|target| (0.95 - (min_dist * 0.01).min(0.5), target))
    };

    if let Some((utility, target)) = evaluate_fight() {
        best_action = ActionType::Fight;
        best_utility = utility;
        best_target = Some(target);
    }

    Some((best_action, best_utility, best_target))
}
