use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of repairing damaged structures.
///
/// Repair is critical for colony survival (preventing building collapse).
/// Thus, it has a slightly higher `base_utility` (0.6) than regular work (0.5).
///
/// This function specifically looks for [`DesignationType::Repair`].
#[must_use]
pub fn evaluate_repair<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: impl Iterator<Item = (Entity, &'a GridPosition, &'a Designation)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Higher priority than normal work

    for (entity, pos, des) in designations {
        if des.designation_type != DesignationType::Repair {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let success = calculate_success_modifier(ActionType::Repair, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}
