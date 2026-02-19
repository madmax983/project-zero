//! Logic for the "Work" action (Mining, Building, etc.).
//!
//! # The Work Action
//!
//! "Work" is a catch-all action for physical labor driven by [`crate::layer1::designation::Designation`]s.
//!
//! ## Scope
//!
//! This action handles:
//! *   **Mining**: Extracting resources from rocks.
//! *   **Forestry**: Chopping trees.
//! *   **Construction**: Building structures (not yet fully implemented in `evaluate_work`, often separate).
//! *   **Demolition**: Removing structures.
//!
//! ## Evaluation Logic
//!
//! The utility of working is calculated by:
//! 1.  **Distance**: Closer designations are preferred.
//! 2.  **Base Utility**: A constant (0.5) baseline.
//! 3.  **Success Modifier**: Adjusted by the pop's history of success/failure with `ActionType::Work`.
//! 4.  **Taboo**: Penalized if the action violates current laws.
//!
//! ## Exclusions
//!
//! *   **Repair**: Handled by [`crate::layer1::actions::repair::evaluate_repair`] (higher priority).
//! *   **Hauling**: Handled by [`crate::layer1::actions::haul::evaluate_haul`].

use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::calculate_context_score;
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing designated work.
///
/// This checks all active designations (pre-filtered for Work types) and calculates
/// a score based on distance and the Pop's work ethic.
///
/// # Examples
///
/// ```rust,ignore
/// use scale::layer1::actions::work::evaluate_work;
/// use scale::layer1::map::GridPosition;
/// use scale::layer1::utility_types::UtilityWeights;
/// use scale::layer1::utility_eval_types::PositionProxy;
/// use bevy_ecs::prelude::*;
///
/// let pop_pos = GridPosition { x: 0, y: 0 };
/// let weights = UtilityWeights::default();
///
/// // Create a dummy designation proxy at (10, 0)
/// let designation = PositionProxy {
///     entity: Entity::PLACEHOLDER,
///     pos: GridPosition { x: 10, y: 0 },
/// };
///
/// let result = evaluate_work(pop_pos, &weights, &[designation]);
///
/// if let Some((utility, entity)) = result {
///     println!("Found work with utility: {}", utility);
///     // Distance 10 -> Context Score ~0.5
///     // Base Utility 0.5
///     // Total ~0.25
///     assert!(utility > 0.2 && utility < 0.3);
/// }
/// ```
///
/// # Returns
/// A tuple `(utility, designation_entity)` if a suitable task is found.
#[must_use]
pub(crate) fn evaluate_work(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    designations: &[PositionProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    // Base utility for working (could depend on traits later)
    let base_utility = 0.5;

    for des in designations {
        // Repair designations are pre-filtered out

        let context = calculate_context_score(
            pop_pos,
            Some(des.pos),
            1, // Capacity 1 (one worker per tile usually)
            0, // Occupied 0 (simplified for now)
            weights,
        );

        let utility = base_utility * context;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, des.entity));
        }
    }
    best
}
