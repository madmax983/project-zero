use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

/// High-level action types pops can choose
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActionType {
    /// Eat food to reduce hunger
    SatisfyHunger,
    /// Sleep to reduce fatigue
    SatisfyRest,
    /// Interact with other pops
    Socialize,
    /// Explore the map
    Explore,
    /// Perform designated work (Mine, Build, Chop)
    Work,
    /// Research new technologies at Library
    Research,
    /// Do nothing
    Idle,
}

impl ActionType {
    /// Total number of action types
    pub const COUNT: usize = 7;

    /// Converts action type to array index
    #[must_use]
    pub const fn as_index(self) -> usize {
        match self {
            Self::SatisfyHunger => 0,
            Self::SatisfyRest => 1,
            Self::Socialize => 2,
            Self::Explore => 3,
            Self::Work => 4,
            Self::Research => 5,
            Self::Idle => 6,
        }
    }
}

/// Pop's current action and commitment state
#[derive(Component, Debug)]
pub struct PopAction {
    /// The current action being performed
    pub current: ActionType,
    /// The utility score of the current action
    pub current_utility: f32,
    /// How many ticks the pop has been doing this action
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

/// Learned utility weights (reinforcement learning)
#[derive(Component, Clone, Copy, Debug)]
pub struct UtilityWeights {
    /// Weight for distance factor (lower distance is better)
    pub distance_weight: f32,
    /// Weight for availability factor (less crowded is better)
    pub availability_weight: f32,
    /// Weight for social factor
    pub social_weight: f32,
    /// Count of successful actions per type
    pub action_success_count: [u32; ActionType::COUNT],
    /// Count of attempted actions per type
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

/// Tracks HTN plan for learning
#[derive(Component, Debug)]
pub struct PlanOutcome {
    /// The action type being tracked
    pub action: ActionType,
    /// Tick when the action started
    pub started_at: u64,
    /// Needs state before the action
    pub needs_before: Needs,
}

/// Global configuration for utility system
#[derive(Resource, Clone)]
pub struct UtilityConfig {
    /// Minimum utility difference required to switch actions
    pub switch_threshold: f32,
    /// How often (in ticks) to re-evaluate actions
    pub evaluation_interval: u32,
    /// How fast weights adjust (0.0 to 1.0)
    pub learning_rate: f32,
    /// Min and max values for weights
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

/// Colony-wide memory (zeitgeist)
#[derive(Resource, Default, Clone)]
pub struct ColonyMemory {
    /// Total successful actions across all pops
    pub total_successful_actions: [u32; ActionType::COUNT],
    /// Average duration of actions
    pub average_action_duration: [u32; ActionType::COUNT],
}

/// Marker component to trigger HTN plan creation
#[derive(Component)]
pub struct StartPlan {
    /// The action to plan for
    pub action: ActionType,
    /// The target entity (if any)
    pub target: Option<Entity>,
}

/// Stub for HTN Plan component (future integration)
#[derive(Component)]
pub struct Plan;
