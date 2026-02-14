use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::map::GridPosition;
use crate::layer1::structure::{DeferMaintenance, Structure};
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;

/// Evaluates the utility of repairing damaged structures.
///
/// Repair is critical for colony survival (preventing building collapse).
/// Thus, it has a slightly higher `base_utility` (0.6) than regular work (0.5).
///
/// This function looks for:
/// 1. Manual [`DesignationType::Repair`].
/// 2. Automatic repair of damaged [`Structure`]s (unless [`DeferMaintenance`] is present).
#[must_use]
pub fn evaluate_repair<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: impl Iterator<Item = (Entity, &'a GridPosition, &'a Designation)>,
    structures: impl Iterator<
        Item = (
            Entity,
            &'a GridPosition,
            &'a Structure,
            Option<&'a DeferMaintenance>,
        ),
    >,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Higher priority than normal work

    // 1. Check Manual Designations
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

    // 2. Check Automatic Repairs (Damaged Structures)
    for (entity, pos, structure, defer) in structures {
        // Skip if maintenance is deferred
        if defer.is_some() {
            continue;
        }

        // Skip if full health
        if (structure.current_hp - structure.max_hp).abs() < f32::EPSILON {
            continue;
        }

        // Automatic repair logic
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
