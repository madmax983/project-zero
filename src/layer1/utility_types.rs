use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
use strum_macros::EnumIter;

/// The menu of high-level behaviors a Pop can choose from.
///
/// These are "Goals" rather than atomic steps. For example, [`ActionType::Work`] implies
/// finding a designation, walking to it, and performing the task until complete.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum ActionType {
    /// Eat food to reduce hunger.
    SatisfyHunger,
    /// Sleep to reduce fatigue.
    SatisfyRest,
    /// Interact with other pops to fulfill social needs.
    Socialize,
    /// Wander to uncover the fog of war or investigate points of interest.
    Explore,
    /// Perform designated physical labor (Mine, Build, Chop).
    Work,
    /// Repair damaged structures to prevent collapse.
    Repair,
    /// Research new technologies at a library.
    Research,
    /// Haul loose resources to a stockpile.
    Haul,
    /// Seek medical care at a hospital.
    SeekMedicalCare,
    /// Bury a corpse in a grave.
    BuryCorpse,
    /// Fetch a tool from a stockpile.
    FetchTool,
    /// Do nothing.
    Idle,
    /// Destroy structures due to mental break.
    Vandalize,
    /// Consume resources uncontrollably due to mental break.
    Binge,
    /// Wander aimlessly in a catatonic state due to mental break.
    Daze,
    /// Engage in combat with hostile entities.
    Fight,
    /// Refine resources at a building (e.g., Lumber Mill).
    Refine,
    /// Work at a farm to produce food.
    Farm,
    /// Arrest Wanted criminals and escort them to jail.
    Warden,
    /// Sleepwalk (Mental Break).
    Sleepwalking,
    /// Tame a wild animal.
    Tame,
    /// Slaughter a tamed animal for resources.
    Slaughter,
}

impl ActionType {
    /// Total number of action types. Used for array sizing.
    pub const COUNT: usize = 22;

    /// Converts action type to a unique array index (0..COUNT-1).
    #[must_use]
    pub const fn as_index(self) -> usize {
        match self {
            Self::SatisfyHunger => 0,
            Self::SatisfyRest => 1,
            Self::Socialize => 2,
            Self::Explore => 3,
            Self::Work => 4,
            Self::Repair => 5,
            Self::Research => 6,
            Self::Haul => 7,
            Self::SeekMedicalCare => 8,
            Self::BuryCorpse => 9,
            Self::FetchTool => 10,
            Self::Idle => 11,
            Self::Vandalize => 12,
            Self::Binge => 13,
            Self::Daze => 14,
            Self::Fight => 15,
            Self::Refine => 16,
            Self::Farm => 17,
            Self::Warden => 18,
            Self::Sleepwalking => 19,
            Self::Tame => 20,
            Self::Slaughter => 21,
        }
    }

    /// Returns the danger level of the action (probability of accident per tick).
    ///
    /// This value is used by the execution systems (e.g., `work_execution_system`)
    /// to determine if a Pop should suffer an injury while performing this task.
    ///
    /// *   **Work/Repair/Slaughter**: 0.1% chance per tick. (Low risk)
    /// *   **Tame**: 0.5% chance per tick. (Moderate risk - animals bite!)
    /// *   **Fight**: 0% (Combat handles damage via its own system).
    /// *   **Others**: 0% chance.
    #[must_use]
    pub const fn danger_level(&self) -> f64 {
        match self {
            Self::Work | Self::Repair | Self::Slaughter => 0.001, // 0.1% chance per tick
            Self::Tame => 0.005, // 0.5% chance per tick (animals bite!)
            _ => 0.0,
        }
    }

    /// Returns the damage inflicted if an accident occurs.
    ///
    /// If `danger_level` triggers an accident, this value determines the amount of HP
    /// lost by the Pop.
    ///
    /// *   **Work/Repair**: 10.0 HP (minor injury).
    /// *   **Tame**: 15.0 HP (animal bite/kick).
    /// *   **Slaughter**: 5.0 HP (accidental cut).
    #[must_use]
    pub const fn accident_damage(&self) -> f32 {
        match self {
            Self::Work | Self::Repair => 10.0,
            Self::Tame => 15.0,
            Self::Slaughter => 5.0,
            _ => 0.0,
        }
    }
}

/// The current state of a Pop's brain.
#[derive(Component, Debug, Clone, Copy)]
pub struct PopAction {
    /// The current action being performed.
    pub current: ActionType,
    /// The utility score of the current action (snapshot at decision time).
    pub current_utility: f32,
    /// How many ticks the pop has been doing this action (commitment timer).
    pub ticks_committed: u32,
}

impl Default for PopAction {
    fn default() -> Self {
        Self {
            current: ActionType::Idle,
            current_utility: 0.0,
            ticks_committed: 0,
        }
    }
}

/// The "Personality" or "Memory" of a Pop (Reinforcement Learning).
///
/// Pops learn from experience. If they successfully find food far away, they might
/// increase their tolerance for distance (`distance_weight` decreases).
/// If they fail to find a spot in a crowded tavern, they become more sensitive
/// to crowding (`availability_weight` increases).
#[derive(Component, Clone, Copy, Debug)]
pub struct UtilityWeights {
    /// How much distance penalties affect scoring.
    /// *   Higher (> 1.0): Hates walking.
    /// *   Lower (< 1.0): Willing to travel.
    pub distance_weight: f32,

    /// How much crowding affects scoring.
    /// *   Higher: Hates crowds (introvert).
    /// *   Lower: Doesn't mind sharing space.
    pub availability_weight: f32,

    /// Preference for social interactions.
    pub social_weight: f32,

    /// Count of successful actions per type (Long-term memory).
    pub action_success_count: [u32; ActionType::COUNT],

    /// Count of attempted actions per type.
    pub action_attempt_count: [u32; ActionType::COUNT],
}

impl Default for UtilityWeights {
    fn default() -> Self {
        Self {
            distance_weight: 1.0,
            availability_weight: 1.0,
            social_weight: 1.0,
            action_success_count: [0; ActionType::COUNT],
            action_attempt_count: [0; ActionType::COUNT],
        }
    }
}

/// Global tuning configuration for the Utility AI system.
#[derive(Resource, Clone)]
pub struct UtilityConfig {
    /// Hysteresis factor to prevent "dithering" (rapidly switching tasks).
    /// A new action must be `current_utility + switch_threshold` better to swap.
    pub switch_threshold: f32,

    /// How often (in ticks) to re-evaluate actions.
    /// Higher = better performance but slower reaction time.
    pub evaluation_interval: u32,

    /// How fast weights adjust (0.0 to 1.0).
    /// *   High: Volatile personality (reacts strongly to recent events).
    /// *   Low: Stubborn personality.
    pub learning_rate: f32,

    /// Min and max values for weights (e.g., 0.5 to 2.0).
    /// Prevents weights from exploding to infinity or zero.
    pub weight_clamp: (f32, f32),
}

impl Default for UtilityConfig {
    fn default() -> Self {
        Self {
            switch_threshold: 0.15,
            evaluation_interval: 1,
            learning_rate: 0.05,
            weight_clamp: (0.5, 2.0),
        }
    }
}

/// Marker component to trigger HTN (Hierarchical Task Network) planning.
///
/// When the Utility AI selects a new high-level [`ActionType`], it inserts this
/// component to tell the planner to generate the specific steps (Walk -> `PickUp` -> Eat).
#[derive(Component)]
pub struct StartPlan {
    /// The action to plan for.
    pub action: ActionType,
    /// The target entity (if any).
    pub target: Option<Entity>,
}

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
/// use scale::layer1::utility_types::need_response_curve;
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
/// use scale::layer1::utility_types::manhattan_distance;
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
/// This score represents "how convenient" a target is.
///
/// # Formula
///
/// 1.  **Distance**: Hyperbolic decay: `1.0 / (1.0 + 0.1 * distance)`.
///     *   At distance 0, score is 1.0.
///     *   At distance 10, score is 0.5 (half utility).
///     *   At distance 90, score is 0.1.
///     *   Raised to the power of `weights.distance_weight`.
///
/// 2.  **Availability**: Linear fraction: `1.0 - (occupied / capacity)`.
///     *   Raised to the power of `weights.availability_weight`.
///
/// # Examples
///
/// ```
/// use scale::layer1::utility_types::{calculate_context_score, UtilityWeights};
/// use scale::layer1::map::GridPosition;
///
/// let pop_pos = GridPosition { x: 0, y: 0 };
/// let farm_pos = GridPosition { x: 10, y: 0 };
/// let weights = UtilityWeights::default();
///
/// // Scenario 1: Empty farm, moderate distance
/// let score_empty = calculate_context_score(
///     pop_pos,
///     Some(farm_pos),
///     10, // Capacity
///     0,  // Occupied
///     &weights
/// );
/// // Distance factor: 1.0 / (1.0 + 0.1*10) = 0.5
/// assert!((score_empty - 0.5).abs() < f32::EPSILON);
///
/// // Scenario 2: Crowded farm (50% full)
/// let score_crowded = calculate_context_score(
///     pop_pos,
///     Some(farm_pos),
///     10,
///     5,  // 5/10 occupied
///     &weights
/// );
/// // Distance (0.5) * Availability (0.5) = 0.25
/// assert!((score_crowded - 0.25).abs() < f32::EPSILON);
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
        if distance > 0 {
            #[allow(clippy::cast_precision_loss)]
            let distance_factor = 1.0 / (distance as f32).mul_add(0.1, 1.0);

            if (weights.distance_weight - 1.0).abs() < f32::EPSILON {
                score *= distance_factor;
            } else {
                score *= distance_factor.powf(weights.distance_weight);
            }
        }
    }

    // Availability factor (less crowded = better)
    if building_capacity > 0 {
        #[allow(clippy::cast_precision_loss)]
        let availability = 1.0 - (building_occupied as f32 / building_capacity as f32);

        if (weights.availability_weight - 1.0).abs() < f32::EPSILON {
            score *= availability;
        } else {
            score *= availability.powf(weights.availability_weight);
        }
    }

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
/// use scale::layer1::utility_types::calculate_success_modifier;
/// use scale::layer1::utility_types::{ActionType, UtilityWeights};
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
    use strum::IntoEnumIterator;

    #[test]
    fn test_action_type_completeness() {
        for action in ActionType::iter() {
            assert!(
                action.as_index() < ActionType::COUNT,
                "Action index out of bounds: {:?}",
                action
            );
        }
    }

    #[test]
    fn test_action_type_count_matches() {
        assert_eq!(
            ActionType::iter().count(),
            ActionType::COUNT,
            "ActionType::COUNT mismatch"
        );
    }

    #[test]
    fn test_need_response_curve_boundaries() {
        // Curve: 1.0 - x^2
        // If x=1.5, 1.0 - 2.25 = -1.25.
        // If x=-0.5, 1.0 - 0.25 = 0.75.
        let val_over = need_response_curve(1.5);
        assert!(val_over < 0.0);

        let val_neg = need_response_curve(-0.5);
        assert!(val_neg < 1.0 && val_neg > 0.0);
    }

    #[test]
    fn test_calculate_context_score_zero_capacity() {
        let weights = UtilityWeights::default();
        let pop_pos = GridPosition { x: 0, y: 0 };
        // Capacity 0 should not panic and return valid score (considering availability logic skipped)
        let score = calculate_context_score(pop_pos, None, 0, 0, &weights);
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_success_modifier_overflow() {
        let mut weights = UtilityWeights::default();
        let idx = ActionType::SatisfyHunger.as_index();

        // Edge case: More successes than attempts (data corruption?)
        weights.action_attempt_count[idx] = 10;
        weights.action_success_count[idx] = 20;

        let mod_val = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        // Rate = 2.0. Modifier = 2.0 * 0.4 + 0.8 = 1.6.
        assert!(mod_val > 1.2);
    }

    #[test]
    fn test_manhattan_distance_extreme() {
        // Test overflow protection
        let min = GridPosition {
            x: i32::MIN,
            y: i32::MIN,
        };
        let max = GridPosition {
            x: i32::MAX,
            y: i32::MAX,
        };

        // Distance should be huge but clamped to i32::MAX
        let dist = manhattan_distance(&min, &max);
        assert_eq!(dist, i32::MAX);
    }

    #[test]
    fn test_manhattan_distance_overflow() {
        let pos1 = GridPosition { x: i32::MIN, y: 0 };
        let pos2 = GridPosition { x: 1, y: 0 };
        let d = manhattan_distance(&pos1, &pos2);
        assert_eq!(d, i32::MAX);
    }

    #[test]
    fn test_action_type_variants() {
        let hunger = ActionType::SatisfyHunger;
        let rest = ActionType::SatisfyRest;
        let idle = ActionType::Idle;

        assert_ne!(hunger, rest);
        assert_ne!(hunger, idle);
        assert_eq!(idle, ActionType::Idle);
    }

    #[test]
    fn test_pop_action_default() {
        let action = PopAction::default();
        assert_eq!(action.current, ActionType::Idle);
        assert!((action.current_utility - 0.0).abs() < f32::EPSILON);
        assert_eq!(action.ticks_committed, 0);
    }

    #[test]
    fn test_utility_weights_default() {
        let weights = UtilityWeights::default();
        assert!((weights.distance_weight - 1.0).abs() < f32::EPSILON);
        assert!((weights.availability_weight - 1.0).abs() < f32::EPSILON);
        assert!((weights.social_weight - 1.0).abs() < f32::EPSILON);
        assert!(weights.action_success_count.iter().all(|&x| x == 0));
        assert!(weights.action_attempt_count.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_need_response_curve() {
        let urgency_high = need_response_curve(0.9);
        assert!(urgency_high < 0.2);

        let urgency_med = need_response_curve(0.5);
        assert!(urgency_med > 0.6 && urgency_med < 0.8);

        let urgency_low = need_response_curve(0.1);
        assert!(urgency_low > 0.95);

        assert!(urgency_low > urgency_med);
        assert!(urgency_med > urgency_high);
    }

    #[test]
    fn test_calculate_context_score_distance() {
        let weights = UtilityWeights::default();

        let close_score = calculate_context_score(
            GridPosition { x: 0, y: 0 },
            Some(GridPosition { x: 1, y: 0 }),
            4,
            0,
            &weights,
        );

        let far_score = calculate_context_score(
            GridPosition { x: 0, y: 0 },
            Some(GridPosition { x: 10, y: 0 }),
            4,
            0,
            &weights,
        );

        assert!(close_score > far_score);
    }

    #[test]
    fn test_calculate_context_score_availability() {
        let weights = UtilityWeights::default();
        let pos = GridPosition { x: 0, y: 0 };

        let empty_score =
            calculate_context_score(pos, Some(GridPosition { x: 5, y: 5 }), 4, 0, &weights);

        let full_score =
            calculate_context_score(pos, Some(GridPosition { x: 5, y: 5 }), 4, 3, &weights);

        assert!(empty_score > full_score);
    }

    #[test]
    fn test_calculate_success_modifier() {
        let mut weights = UtilityWeights::default();

        let neutral = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(neutral > 0.9 && neutral < 1.1);

        weights.action_attempt_count[ActionType::SatisfyHunger.as_index()] = 10;
        weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 9;
        let high = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(high > 1.0);

        weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 2;
        let low = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(low < 1.0);
    }

    #[test]
    fn test_manhattan_distance() {
        let pos1 = GridPosition { x: 0, y: 0 };
        let pos2 = GridPosition { x: 3, y: 4 };
        assert_eq!(manhattan_distance(&pos1, &pos2), 7);

        let pos3 = GridPosition { x: -2, y: 5 };
        assert_eq!(manhattan_distance(&pos1, &pos3), 7);
    }

    #[test]
    fn test_utility_config_default() {
        let config = UtilityConfig::default();
        assert!((config.switch_threshold - 0.15).abs() < f32::EPSILON);
        assert_eq!(config.evaluation_interval, 1);
        assert!((config.learning_rate - 0.05).abs() < f32::EPSILON);
        assert_eq!(config.weight_clamp, (0.5, 2.0));
    }
}
