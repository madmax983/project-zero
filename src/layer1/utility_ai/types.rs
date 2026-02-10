//! Core types for the Utility AI system.
//!
//! This module defines the vocabulary of the AI:
//! *   **Actions**: What can a Pop do? ([`ActionType`])
//! *   **State**: What is the Pop doing now? ([`PopAction`])
//! *   **Memory**: What has the Pop learned? ([`UtilityWeights`])
//! *   **Config**: Global tuning knobs ([`UtilityConfig`])

use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

/// The menu of high-level behaviors a Pop can choose from.
///
/// These are "Goals" rather than atomic steps. For example, `Work` implies
/// finding a designation, walking to it, and performing the task until complete.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActionType {
    /// Eat food to reduce hunger.
    ///
    /// Triggered when `hunger` need is low. Requires access to a [`crate::layer1::farm::Farm`]
    /// or food items.
    SatisfyHunger,

    /// Sleep to reduce fatigue.
    ///
    /// Triggered when `rest` need is low. Requires a [`crate::layer1::housing::Housing`] bed
    /// or sleeping on the ground (lower utility).
    SatisfyRest,

    /// Interact with other pops to fulfill social needs.
    ///
    /// Usually happens in a [`crate::layer1::social::Tavern`].
    Socialize,

    /// Wander to uncover the fog of war or investigate points of interest.
    Explore,

    /// Perform designated physical labor (Mine, Build, Chop).
    ///
    /// The specific task depends on the [`crate::layer1::designation::Designation`] target.
    Work,

    /// Repair damaged structures to prevent collapse.
    ///
    /// Higher priority than `Work` to preserve infrastructure.
    Repair,

    /// Research new technologies at a [`crate::layer1::tech::Library`].
    ///
    /// Generates knowledge points.
    Research,

    /// Haul loose resources to a [`crate::layer1::stockpile::Stockpile`].
    ///
    /// Keeps the map clean and consolidates resources.
    Haul,

    /// Seek medical care at a [`crate::layer1::medical::Hospital`].
    ///
    /// Triggered when health is damaged.
    SeekMedicalCare,

    /// Bury a corpse in a grave.
    ///
    /// Triggered when a corpse exists and an empty grave is available.
    BuryCorpse,

    /// Fetch a tool from a stockpile.
    ///
    /// Triggered when a pop has no tool but tools are available in the colony.
    FetchTool,

    /// Do nothing.
    ///
    /// The fallback action when no other options are viable or beneficial.
    Idle,

    /// Destroy structures due to mental break.
    Vandalize,

    /// Consume resources uncontrollably due to mental break.
    Binge,

    /// Wander aimlessly in a catatonic state due to mental break.
    Daze,

    /// Engage in combat with hostile entities.
    Fight,

    /// Tame a wild animal.
    Tame,

    /// Slaughter a tamed animal for resources.
    Slaughter,
}

impl ActionType {
    /// Total number of action types. Used for array sizing.
    pub const COUNT: usize = 18;

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
            Self::Tame => 16,
            Self::Slaughter => 17,
        }
    }

    /// Returns the danger level of the action (probability of accident per tick).
    ///
    /// *   Work/Repair: 0.1% chance.
    /// *   Fight: 0% (Combat handles damage differently).
    /// *   Others: 0% chance.
    #[must_use]
    pub const fn danger_level(&self) -> f64 {
        match self {
            Self::Work | Self::Repair => 0.001, // 0.1% chance per tick
            _ => 0.0,
        }
    }

    /// Returns the damage inflicted if an accident occurs.
    #[must_use]
    pub const fn accident_damage(&self) -> f32 {
        match self {
            Self::Work | Self::Repair => 10.0,
            _ => 0.0,
        }
    }
}

/// The current state of a Pop's brain.
#[derive(Component, Debug)]
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

/// Tracks the outcome of a plan for learning purposes.
///
/// When an action completes (success or failure), this component is used
/// to update the [`UtilityWeights`].
#[derive(Component, Debug)]
pub struct PlanOutcome {
    /// The action type being tracked.
    pub action: ActionType,
    /// Tick when the action started.
    pub started_at: u64,
    /// Needs state before the action (to measure improvement).
    pub needs_before: Needs,
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

/// Colony-wide memory (zeitgeist).
///
/// Tracks aggregate statistics about action success rates across all pops.
/// (Currently used for debugging/inspector).
#[derive(Resource, Default, Clone)]
pub struct ColonyMemory {
    /// Total successful actions across all pops.
    pub total_successful_actions: [u32; ActionType::COUNT],
    /// Average duration of actions (in ticks).
    pub average_action_duration: [u32; ActionType::COUNT],
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

/// Stub for HTN Plan component (future integration).
#[derive(Component)]
pub struct Plan;
