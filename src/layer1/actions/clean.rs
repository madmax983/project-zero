use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::{calculate_context_score, UtilityWeights};
use bevy_ecs::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::Entity;

    #[test]
    fn test_evaluate_clean_empty_candidates() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let candidates = vec![];

        let result = evaluate_clean(pop_pos, &weights, &candidates, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_clean_janitor_selects_all_clutter() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let mut cand_low = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 1, y: 0 });
        cand_low.score_bonus = 0.5; // low clutter

        let candidates = vec![cand_low];

        let result = evaluate_clean(pop_pos, &weights, &candidates, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, Entity::from_raw(1));
    }

    #[test]
    fn test_evaluate_clean_non_janitor_skips_low_clutter() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let mut cand_low = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 1, y: 0 });
        cand_low.score_bonus = 0.5; // low clutter

        let candidates = vec![cand_low];

        let result = evaluate_clean(pop_pos, &weights, &candidates, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_clean_non_janitor_selects_critical_clutter() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let mut cand_critical =
            ScorableCandidate::new(Entity::from_raw(2), GridPosition { x: 1, y: 0 });
        cand_critical.score_bonus = 0.9; // critical clutter

        let candidates = vec![cand_critical];

        let result = evaluate_clean(pop_pos, &weights, &candidates, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, Entity::from_raw(2));
    }

    #[test]
    fn test_evaluate_clean_selects_best_candidate() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        // Use default weights which likely factor in distance.
        // If distance matters, a closer candidate has higher context score.
        let weights = UtilityWeights::default();

        let mut cand_far_low =
            ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 10, y: 0 });
        cand_far_low.score_bonus = 0.2;

        let mut cand_close_high =
            ScorableCandidate::new(Entity::from_raw(2), GridPosition { x: 1, y: 0 });
        cand_close_high.score_bonus = 0.9;

        let candidates = vec![cand_far_low, cand_close_high];

        // For janitor, both are considered, but cand_close_high should have much higher utility
        let result = evaluate_clean(pop_pos, &weights, &candidates, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, Entity::from_raw(2));
    }
}
