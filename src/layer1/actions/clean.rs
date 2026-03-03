use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::{calculate_context_score, UtilityWeights};

/// Evaluates the utility of cleaning clutter.
#[must_use]
pub(crate) fn evaluate_clean(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    is_janitor: bool,
) -> Option<(f32, Entity)> {
    if candidates.is_empty() {
        return None;
    }

    // Janitors prioritize cleaning. Idle pops do it as community service if bad enough.
    // Base utility:
    // Janitor: 0.6 (Good priority)
    // Idle: 0.1 (Low priority, requires critical clutter)

    let base_utility = if is_janitor { 0.6 } else { 0.1 };

    // Find best target.
    // Score bonus in candidate is clutter amount / 100.0.
    // So 80.0 clutter = 0.8 bonus.
    // Janitor: 0.6 + 0.8 = 1.4 (Very high)
    // Idle: 0.1 + 0.8 = 0.9 (High enough to beat Idle 0.05)
    //
    // If clutter is 40.0:
    // Janitor: 0.6 + 0.4 = 1.0
    // Idle: 0.1 + 0.4 = 0.5 (Might beat explore/idle)

    // Filter for non-janitors: Only accept if clutter > 0.8 (Critical)
    // We can filter candidates or just rely on scoring.
    // Let's rely on scoring but enforce a threshold for non-janitors.

    let mut best: Option<(f32, Entity)> = None;

    for candidate in candidates {
        // Skip if not janitor and clutter not critical
        if !is_janitor && candidate.score_bonus < 0.8 {
            continue;
        }

        let context = calculate_context_score(
            pop_pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            weights,
        );

        let utility = (base_utility + candidate.score_bonus) * context;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, candidate.entity));
        }
    }
    best
}
