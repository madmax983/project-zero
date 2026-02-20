//! Logic for the "Admin" action (Working in an Office).

use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of working as an Administrator.
#[must_use]
pub fn evaluate_admin(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    offices: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, offices, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_admin_finds_best_office() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let office1 = ScorableCandidate::with_capacity(
            Entity::from_raw(1),
            GridPosition { x: 10, y: 0 },
            2,
            0,
        );

        let office2 = ScorableCandidate::with_capacity(
            Entity::from_raw(2),
            GridPosition { x: 1, y: 0 }, // Closer
            2,
            0,
        );

        let result = evaluate_admin(pop_pos, &weights, &[office1, office2.clone()]);
        assert!(result.is_some());
        let (_, entity) = result.unwrap();
        assert_eq!(entity, office2.entity);
    }
}
