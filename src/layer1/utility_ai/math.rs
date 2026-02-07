use super::types::{ActionType, UtilityWeights};
use crate::layer1::map::GridPosition;

/// Calculates urgency from a need value (0.0-1.0).
/// Lower need value = higher urgency.
#[must_use]
pub fn need_response_curve(need_value: f32) -> f32 {
    need_value.mul_add(-need_value, 1.0)
}

/// Calculates Manhattan distance between two positions.
#[must_use]
pub const fn manhattan_distance(pos1: &GridPosition, pos2: &GridPosition) -> i32 {
    let dx = (pos1.x as i64 - pos2.x as i64).abs();
    let dy = (pos1.y as i64 - pos2.y as i64).abs();
    let sum = dx + dy;
    if sum > i32::MAX as i64 {
        i32::MAX
    } else {
        sum as i32
    }
}

/// Calculates a score based on context (distance, availability).
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
