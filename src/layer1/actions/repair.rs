//! Logic for the "Repair" action.
//!
//! # The Repair Action
//!
//! Maintenance is key to preventing structural collapse.
//!
//! ## Modes
//!
//! 1.  **Manual Repair**: Triggered by a [`crate::layer1::designation::DesignationType::Repair`].
//!     *   Highest priority.
//!     *   Player-directed.
//! 2.  **Automatic Maintenance**: Triggered when a structure is damaged (HP < Max HP).
//!     *   Automatic (Pops find it themselves).
//!     *   Can be disabled via [`crate::layer1::structure::DeferMaintenance`].
//!
//! ## Priority
//!
//! Repair has a higher base utility (0.6) than standard Work (0.5), reflecting the urgency of safety.

use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::utility_types::UtilityWeights;
use crate::layer1::utility_types::calculate_context_score;
use bevy_ecs::prelude::*;

/// Evaluates the utility of repairing damaged structures.
///
/// Repair is critical for colony survival (preventing building collapse).
/// Thus, it has a slightly higher `base_utility` (0.6) than regular work (0.5).
///
/// This function looks for:
/// 1. Manual [`crate::layer1::designation::DesignationType::Repair`].
/// 2. Automatic repair of damaged [`crate::layer1::structure::Structure`]s (unless [`crate::layer1::structure::DeferMaintenance`] is present).
///
/// # Examples
///
/// ```rust,ignore
/// use scale::layer1::actions::repair::evaluate_repair;
/// use scale::layer1::map::GridPosition;
/// use scale::layer1::utility_types::UtilityWeights;
/// use scale::layer1::utility_eval_types::PositionProxy;
/// use bevy_ecs::prelude::*;
///
/// let weights = UtilityWeights::default();
/// let pop_pos = GridPosition { x: 0, y: 0 };
///
/// // A damaged wall at (5, 0)
/// let wall_proxy = PositionProxy {
///     entity: Entity::PLACEHOLDER,
///     pos: GridPosition { x: 5, y: 0 },
/// };
///
/// let result = evaluate_repair(pop_pos, &weights, &[], &[wall_proxy]);
///
/// if let Some((utility, entity)) = result {
///     // Base (0.6) * Distance Factor (~0.66) * Success (1.0) ~= 0.4
///     println!("Repair utility: {}", utility);
///     assert!(utility > 0.3);
/// }
/// ```
#[must_use]
pub(crate) fn evaluate_repair(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    designations: &[PositionProxy],
    structures: &[PositionProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Higher priority than normal work

    // 1. Check Manual Designations (Pre-filtered for Repair type)
    for des in designations {
        let context = calculate_context_score(
            pop_pos,
            Some(des.pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let utility = base_utility * context;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, des.entity));
        }
    }

    // 2. Check Automatic Repairs (Damaged Structures)
    // Pre-filtered for !DeferMaintenance and Damaged status
    for structure in structures {
        // Automatic repair logic
        let context = calculate_context_score(
            pop_pos,
            Some(structure.pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let utility = base_utility * context;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, structure.entity));
        }
    }

    best
}
