use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::{RepairDesignationProxy, StructureProxy};
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;

/// Evaluates the utility of repairing damaged structures.
///
/// Repair is critical for colony survival (preventing building collapse).
/// Thus, it has a slightly higher `base_utility` (0.6) than regular work (0.5).
///
/// This function looks for:
/// 1. Manual [`crate::layer1::designation::DesignationType::Repair`].
/// 2. Automatic repair of damaged [`crate::layer1::structure::Structure`]s (unless [`crate::layer1::structure::DeferMaintenance`] is present).
#[must_use]
pub fn evaluate_repair(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: &[RepairDesignationProxy],
    structures: &[StructureProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Higher priority than normal work

    // 1. Check Manual Designations (Pre-filtered for Repair type)
    for des in designations {
        let context = calculate_context_score(
            *pop_pos,
            Some(des.pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let success = calculate_success_modifier(ActionType::Repair, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, des.entity));
        }
    }

    // 2. Check Automatic Repairs (Damaged Structures)
    // Pre-filtered for !DeferMaintenance and Damaged status
    for structure in structures {
        // Automatic repair logic
        let context = calculate_context_score(
            *pop_pos,
            Some(structure.pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let success = calculate_success_modifier(ActionType::Repair, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, structure.entity));
        }
    }

    best
}
