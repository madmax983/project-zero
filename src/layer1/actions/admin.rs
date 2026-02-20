//! Logic for the "Admin" action (Working in an Office).

use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::CapacityProxy;
use crate::layer1::utility_types::{UtilityWeights, calculate_context_score};
use bevy_ecs::prelude::*;

/// Evaluates the utility of working as an Administrator.
///
/// # Returns
/// A tuple `(utility, target_entity)` if a suitable workplace is found.
#[must_use]
pub fn evaluate_admin(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    offices: &[CapacityProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    for office in offices {
        let context = calculate_context_score(
            pop_pos,
            Some(office.pos),
            office.capacity,
            office.usage,
            weights,
        );

        let utility = base_utility * context;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, office.entity));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::utility_eval_types::CapacityProxy;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_admin_finds_best_office() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let office1 = CapacityProxy {
            entity: Entity::from_raw(1),
            pos: GridPosition { x: 10, y: 0 },
            capacity: 2,
            usage: 0,
        };

        let office2 = CapacityProxy {
            entity: Entity::from_raw(2),
            pos: GridPosition { x: 1, y: 0 }, // Closer
            capacity: 2,
            usage: 0,
        };

        let result = evaluate_admin(pop_pos, &weights, &[office1, office2]);
        assert!(result.is_some());
        let (_, entity) = result.unwrap();
        assert_eq!(entity, office2.entity);
    }
}
