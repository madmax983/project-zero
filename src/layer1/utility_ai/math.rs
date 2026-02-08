//! Mathematical primitives for utility calculation.
//!
//! This module contains the "Response Curves" and scoring functions used to normalize
//! inputs (needs, distance) into a 0.0-1.0 utility score.

use super::types::{ActionType, UtilityWeights};
use crate::layer1::map::GridPosition;

/// Calculates urgency from a need value (0.0-1.0).
///
/// **Formula**: `1.0 - (need_value^2)`
///
/// This creates a quadratic "urgency curve":
/// *   **High Need** (0.9): Urgency is low (~0.19). You're fine.
/// *   **Low Need** (0.1): Urgency is very high (~0.99). You're starving.
/// *   **Middle** (0.5): Urgency is moderate (0.75).
///
/// This curve prevents Pops from reacting too early to minor hunger, but makes them
/// panic as they get closer to 0.
///
/// # Examples
///
/// ```
/// use scale::layer1::utility_ai::math::need_response_curve;
///
/// let urgency = need_response_curve(0.9); // Full belly
/// assert!((urgency - 0.19).abs() < 0.0001);
///
/// let panic = need_response_curve(0.1); // Starving
/// assert!(panic > 0.9);
/// ```
#[must_use]
pub fn need_response_curve(need_value: f32) -> f32 {
    need_value.mul_add(-need_value, 1.0)
}

/// Calculates Manhattan distance between two positions.
///
/// `|x1 - x2| + |y1 - y2|`
///
/// Safe against overflow (clamps to `i32::MAX`).
///
/// # Examples
///
/// ```
/// use scale::layer1::utility_ai::math::manhattan_distance;
/// use scale::layer1::map::GridPosition;
///
/// let start = GridPosition { x: 0, y: 0 };
/// let end = GridPosition { x: 3, y: 4 };
///
/// assert_eq!(manhattan_distance(&start, &end), 7);
/// ```
#[must_use]
pub const fn manhattan_distance(pos1: &GridPosition, pos2: &GridPosition) -> i32 {
    let dx = (pos1.x as i64 - pos2.x as i64).abs();
    let dy = (pos1.y as i64 - pos2.y as i64).abs();
    let sum = dx + dy;
    if sum > i32::MAX as i64 {
        i32::MAX
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            sum as i32
        }
    }
}

/// Calculates a context score (0.0 - 1.0) based on distance and crowding.
///
/// # Formula
///
/// 1.  **Distance**: Hyperbolic decay: `1.0 / (1.0 + 0.1 * distance)`.
///     *   At distance 0, score is 1.0.
///     *   At distance 10, score is 0.5.
///     *   At distance 90, score is 0.1.
///     *   Raised to the power of `weights.distance_weight`.
///
/// 2.  **Availability**: Linear fraction: `1.0 - (occupied / capacity)`.
///     *   Raised to the power of `weights.availability_weight`.
///
/// # Examples
///
/// ```
/// use scale::layer1::utility_ai::math::calculate_context_score;
/// use scale::layer1::utility_ai::types::UtilityWeights;
/// use scale::layer1::map::GridPosition;
///
/// let pop_pos = GridPosition { x: 0, y: 0 };
/// let farm_pos = GridPosition { x: 10, y: 0 }; // Distance 10
/// let weights = UtilityWeights::default(); // Weight 1.0
///
/// let score = calculate_context_score(
///     pop_pos,
///     Some(farm_pos),
///     10, // Capacity
///     0,  // Occupied (empty)
///     &weights
/// );
///
/// // Distance factor: 1.0 / (1.0 + 0.1*10) = 0.5
/// // Availability: 1.0
/// assert!((score - 0.5).abs() < f32::EPSILON);
/// ```
#[must_use]
pub fn calculate_context_score(
    pop_pos: GridPosition,
    target_pos: Option<GridPosition>,
    building_capacity: usize,
    building_occupied: usize,
    weights: &UtilityWeights,
) -> f32 {
    let mut score = 1.0;

    // Distance factor (closer = better)
    if let Some(target) = target_pos {
        let distance = manhattan_distance(&pop_pos, &target);
        #[allow(clippy::cast_precision_loss)]
        let distance_factor = 1.0 / (distance as f32).mul_add(0.1, 1.0);
        score *= distance_factor.powf(weights.distance_weight);
    }

    // Availability factor (less crowded = better)
    if building_capacity > 0 {
        #[allow(clippy::cast_precision_loss)]
        let availability = 1.0 - (building_occupied as f32 / building_capacity as f32);
        score *= availability.powf(weights.availability_weight);
    }

    // Social factor (future - for now just identity)
    score *= 1.0_f32.powf(weights.social_weight);

    score.clamp(0.0, 1.0)
}

/// Calculates a modifier based on past success rates.
///
/// Returns a multiplier between **0.8** (failure prone) and **1.2** (reliable).
///
/// *   **New Action**: Returns 1.0 (neutral).
/// *   **High Success**: Approaches 1.2.
/// *   **High Failure**: Approaches 0.8.
///
/// # Examples
///
/// ```
/// use scale::layer1::utility_ai::math::calculate_success_modifier;
/// use scale::layer1::utility_ai::types::{ActionType, UtilityWeights};
///
/// let mut weights = UtilityWeights::default();
///
/// // Simulate 100% success rate
/// weights.action_attempt_count[ActionType::SatisfyHunger.as_index()] = 10;
/// weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 10;
///
/// let modifier = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
/// assert!((modifier - 1.2).abs() < 0.0001);
/// ```
#[must_use]
pub fn calculate_success_modifier(action: ActionType, weights: &UtilityWeights) -> f32 {
    let idx = action.as_index();
    let attempts = weights.action_attempt_count[idx];
    let successes = weights.action_success_count[idx];

    if attempts == 0 {
        return 1.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let success_rate = successes as f32 / attempts as f32;

    // Convert to modifier: 0.8-1.2 range
    // Linear interpolation: rate * 0.4 + 0.8
    success_rate.mul_add(0.4, 0.8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_manhattan_distance_overflow() {
        let pos1 = GridPosition { x: i32::MIN, y: 0 };
        let pos2 = GridPosition { x: 1, y: 0 };
        // Original implementation panicked here: (-2147483648 - 1).abs() overflow
        let d = manhattan_distance(&pos1, &pos2);
        // Correct distance is |-2147483648 - 1| = |-2147483649| = 2147483649
        // Clamped to i32::MAX (2147483647)
        assert_eq!(d, i32::MAX);
    }
}
