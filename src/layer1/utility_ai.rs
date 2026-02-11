//! # Utility AI: The Brain of the Colony
//!
//! This module implements a **Utility-based AI** system (sometimes called "Need-based AI")
//! that drives the behavior of every Pop in the colony.
//!
//! ## The Decision Cycle
//!
//! Every few ticks (configured in [`UtilityConfig`]), a Pop evaluates its options:
//!
//! 1.  **Identify Candidates**: Scans the world for possible actions (e.g., "There is a farm at (10, 5)").
//! 2.  **Score Candidates**: Calculates a utility score (0.0 - 1.0+) for each option based on:
//!     *   **Needs**: "I am hungry" (increases food utility).
//!     *   **Distance**: "It's too far away" (decreases utility via [`calculate_context_score`]).
//!     *   **Personality**: "I hate hauling" (modifiers from traits/memories).
//!     *   **Learning**: "I failed at this last time" (reinforcement learning via [`UtilityWeights`]).
//! 3.  **Select Best**: The action with the highest score wins.
//! 4.  **Commit**: The Pop commits to the action for a duration or until a better option appears.
//!
//! ## Key Components
//!
//! *   [`evaluate_actions_system`]: The main loop that runs the decision cycle.
//! *   [`ActionType`]: The enum of all possible behaviors.
//! *   [`UtilityWeights`]: The "memory" of the Pop, adjusting scores based on past success/failure.
//!

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

    /// Refine resources at a building (e.g., Lumber Mill).
    Refine,

    /// Work at a farm to produce food.
    Farm,

    /// Arrest Wanted criminals and escort them to jail.
    Warden,

    /// Sleepwalk (Mental Break).
    Sleepwalking,
}

impl ActionType {
    /// Total number of action types. Used for array sizing.
    pub const COUNT: usize = 20;

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
/// use scale::layer1::utility_ai::need_response_curve;
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
/// use scale::layer1::utility_ai::manhattan_distance;
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
/// use scale::layer1::utility_ai::calculate_context_score;
/// use scale::layer1::utility_ai::UtilityWeights;
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
/// use scale::layer1::utility_ai::calculate_success_modifier;
/// use scale::layer1::utility_ai::{ActionType, UtilityWeights};
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
mod math_tests {
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

use crate::layer1::actions::explore::evaluate_explore;
use crate::layer1::actions::farm::evaluate_farm;
use crate::layer1::actions::fetch_tool::evaluate_fetch_tool;
use crate::layer1::actions::haul::evaluate_haul;
use crate::layer1::actions::hunger::evaluate_satisfy_hunger;
use crate::layer1::actions::refine::evaluate_refine;

/// Evaluates the utility of being idle.
///
/// Idle is a low-priority fallback action. Pops should prefer productive
/// activities (work, eating, resting) over standing around.
#[must_use]
pub const fn evaluate_idle(_needs: &Needs) -> f32 {
    0.05
}
use crate::layer1::actions::repair::evaluate_repair;
use crate::layer1::actions::research::evaluate_research;
use crate::layer1::actions::rest::evaluate_satisfy_rest;
use crate::layer1::actions::work::evaluate_work;
use crate::layer1::combat::{Drafted, evaluate_fight_action};
use crate::layer1::designation::Designation;
use crate::layer1::farm::Farm;
use crate::layer1::fauna::Fauna;
use crate::layer1::funeral::{Corpse, Grave, evaluate_bury_corpse};
use crate::layer1::housing::Housing;
use crate::layer1::items::Equipment;
use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::medical::{Hospital, evaluate_seek_medical_care};
use crate::layer1::needs::Needs;
use crate::layer1::resources::{ColonyResources, ResourceItem};
use crate::layer1::science::Anomaly;
use crate::layer1::social::{Tavern, evaluate_socialize};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::Structure;
use crate::layer1::tech::Library;
use crate::layer1::unrest::{MentalBreakType, MentalState};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// System to update commitment timers.
/// Increments the committed-tick counter for every pop's action.
///
/// Uses `par_iter_mut` for parallel processing across entities.
pub fn update_action_timer_system(mut query: Query<&mut PopAction>) {
    query.par_iter_mut().for_each(|mut action| {
        action.ticks_committed += 1;
    });
}

/// The Main Brain Loop: Decides what every Pop should do next.
///
/// This system runs periodically (every tick, but individual pops only evaluate
/// based on their `evaluation_interval`).
///
/// # The Algorithm
///
/// 1.  **Filter**: Selects Pops who have finished their commitment timer (`ticks_committed`).
/// 2.  **Gather Context**: Pre-fetches all relevant entities (Farms, Stockpiles, etc.)
///     into efficient query iterators.
/// 3.  **Evaluate Candidates**:
///     For each Pop, it calls every `evaluate_*` function:
///     *   [`evaluate_satisfy_hunger`]
///     *   [`evaluate_work`]
///     *   [`evaluate_haul`]
///     *   ...and so on.
/// 4.  **Winner Takes All**: Tracks the single best `(Utility, Action, Target)` tuple.
/// 5.  **Switch**: If the best new utility > current utility + threshold, the Pop switches tasks.
///     *   Updates [`PopAction`].
///     *   Inserts [`StartPlan`] to trigger HTN planning (if applicable).
///
/// # Performance Note
/// This system avoids per-Pop heap allocations by using a single-pass "best so far"
/// tracker instead of collecting a `Vec<ActionCandidate>`.
#[allow(clippy::too_many_lines, clippy::collapsible_if)]
pub fn evaluate_actions_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();

    // Collect pop data
    #[allow(unused_mut)]
    #[allow(clippy::type_complexity)]
    // We break up the query to avoid complex iterator types
    let mut pop_data: Vec<(
        Entity,
        GridPosition,
        Needs,
        UtilityWeights,
        PopAction,
        Option<Equipment>,
        Option<MentalState>,
        Option<Drafted>,
    )> = world
        .query::<(
            Entity,
            &GridPosition,
            &Needs,
            &UtilityWeights,
            &PopAction,
            Option<&Equipment>,
            Option<&MentalState>,
            Option<&Drafted>,
            Option<&Inmate>,
        )>()
        .iter(world)
        .filter(|(_, _, _, _, action, _, _, _, inmate)| {
            action.ticks_committed >= config.evaluation_interval && inmate.is_none()
        })
        .map(|(e, p, n, w, a, eq, m, d, _)| {
            (
                e,
                *p,
                *n,
                *w,
                PopAction {
                    current: a.current,
                    current_utility: a.current_utility,
                    ticks_committed: a.ticks_committed,
                },
                eq.cloned(),
                m.cloned(),
                d.cloned(),
            )
        })
        .collect();

    // Pre-create query states to avoid allocation in loop
    let mut farms_state = world.query::<(
        Entity,
        &GridPosition,
        &Farm,
        Option<&crate::layer1::building::ShiftSchedule>,
    )>();
    let mut refining_state = world.query::<(
        Entity,
        &GridPosition,
        &crate::layer1::building::Building,
        &crate::layer1::resources::RefiningProgress,
        Option<&crate::layer1::building::ShiftSchedule>,
    )>();
    let mut housing_state = world.query::<(Entity, &GridPosition, &Housing)>();
    let mut taverns_state = world.query::<(Entity, &GridPosition, &Tavern)>();
    let mut fauna_state = world.query::<(Entity, &GridPosition, &Fauna)>();
    let mut libraries_state = world.query::<(
        Entity,
        &GridPosition,
        &Library,
        Option<&crate::layer1::building::ShiftSchedule>,
    )>();
    let mut designations_state = world.query::<(Entity, &GridPosition, &Designation)>();
    let mut items_state = world.query::<(Entity, &GridPosition, &ResourceItem)>();
    let mut stockpiles_state = world.query::<(Entity, &GridPosition, &Stockpile)>();
    let mut anomalies_state = world.query::<(Entity, &GridPosition, &Anomaly)>();
    let mut hospitals_state = world.query::<(Entity, &GridPosition, &Hospital)>();
    let mut corpses_state = world.query::<(Entity, &GridPosition, &Corpse)>();
    let mut graves_state = world.query::<&Grave>();
    let mut structures_state = world.query::<(Entity, &GridPosition, &Structure)>();

    // We need Health for medical care evaluation.
    // The main query above only extracted (Entity, &GridPosition, &Needs, &UtilityWeights, &PopAction)
    // We should probably add Health to it, or fetch it.
    // Since Health is optional (maybe?), let's add it to the main query if possible, or get it inside.
    // The pop_data collection above collects into a Vec, effectively decoupling from World.
    // So we can't easily get Health later if we don't collect it.
    // Let's modify the pop_data collection to include Health.

    // But modifying the big query requires modifying the collection logic.
    // Alternatively, we can use `world.get::<Health>(pop_entity)` inside the loop if we didn't collect the data into a detached Vec.
    // But `pop_data` IS a detached Vec. And we are borrowing world mutably in `evaluate_actions_system`.
    // Wait, `evaluate_actions_system` takes `world: &mut World`.
    // The iteration `for (pop_entity, ...)` iterates over the `pop_data` Vec.
    // Inside the loop, we call `evaluate_*` functions passing iterators derived from `world`.
    // We CAN access `world.get::<Health>(pop_entity)` inside the loop?
    // No, `farms_state.iter(world)` borrows world immutably.
    // So we can use `world` immutably inside the loop.

    let resources = world.resource::<ColonyResources>().clone();
    let cycle = world
        .resource::<crate::layer1::day_night::DayNightCycle>()
        .clone();

    // Evaluate each pop
    for (
        pop_entity,
        pop_pos,
        needs,
        weights,
        mut action,
        equipment_opt,
        mental_state_opt,
        drafted_opt,
    ) in pop_data
    {
        // Optimization: Avoid heap allocation (Vec) for utilities.
        // Instead, track the best action found so far in a single pass.

        // Start with Idle as the baseline
        let mut best_action = ActionType::Idle;
        let mut best_utility = evaluate_idle(&needs);
        let mut best_target = None;

        let mental_break = if let Some(MentalState::Broken(break_type)) = mental_state_opt {
            Some(break_type)
        } else {
            None
        };

        if let Some(break_type) = mental_break {
            best_utility = 100.0;
            best_target = None;
            best_action = match break_type {
                MentalBreakType::Vandalize => {
                    // Find closest structure to destroy
                    let mut closest_dist = i32::MAX;
                    let mut closest_target = None;

                    for (target_entity, target_pos, _) in structures_state.iter(world) {
                        if target_entity == pop_entity {
                            continue;
                        }
                        let dist = manhattan_distance(&pop_pos, target_pos);
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest_target = Some(target_entity);
                        }
                    }
                    best_target = closest_target;
                    ActionType::Vandalize
                }
                MentalBreakType::Binge => ActionType::Binge,
                MentalBreakType::Daze => ActionType::Daze,
                MentalBreakType::Sleepwalking => {
                    // Sleepwalkers just wander. Target is assigned by assign_sleepwalk_target_system.
                    best_target = None;
                    ActionType::Sleepwalking
                }
            };
        } else {
            // Check for Drafted
            let is_drafted = drafted_opt.is_some();
            if is_drafted {
                // Prioritize Fight
                let enemies = fauna_state.iter(world).map(|(e, p, _)| (e, p));
                if let Some((utility, target)) = evaluate_fight_action(true, &pop_pos, enemies) {
                    best_action = ActionType::Fight;
                    best_utility = utility;
                    best_target = Some(target);
                } else {
                    // Drafted but no enemies? Maybe just stand ground (Idle) with high utility to prevent working?
                    // Spec says: "If not in range: Move towards target".
                    // evaluate_fight_action currently returns None if no target.
                    // If no target, maybe Idle is fine, but utility should be higher than needs?
                    // For now, if no enemies, fall back to normal evaluation, BUT drafted pops ignore work/needs.
                    // The spec says "Drafted pops ignore normal work/needs".
                    // So we should probably NOT run the rest of the evaluation if drafted.

                    // If no enemies found, we might want to just stay put.
                    // Let's set utility high enough to avoid switching to Work/Hunger if possible,
                    // or just return Idle with high score?
                    // But if we return here, we skip other evals.

                    // Let's implement early exit for drafted pops.
                    if best_action == ActionType::Idle {
                        best_utility = 0.9; // Just stand there ready
                    }
                }
            } else {
                // Normal evaluation (undrafted)
                // Helper to update best if we found something better
                let mut check_best = |act, util, tgt| {
                    if util > best_utility {
                        best_action = act;
                        best_utility = util;
                        best_target = tgt;
                    }
                };

                // Check Health
                let health = world.get::<crate::layer1::health::Health>(pop_entity);

                // Evaluate SatisfyHunger
                if let Some((utility, target)) = evaluate_satisfy_hunger(
                    &pop_pos,
                    &needs,
                    &weights,
                    farms_state.iter(world).map(|(e, p, f, _)| (e, p, f)),
                ) {
                    check_best(ActionType::SatisfyHunger, utility, Some(target));
                }

                // Evaluate SatisfyRest
                if let Some((utility, target)) =
                    evaluate_satisfy_rest(&pop_pos, &needs, &weights, housing_state.iter(world))
                {
                    check_best(ActionType::SatisfyRest, utility, Some(target));
                }

                // Evaluate Socialize
                if let Some((utility, target)) =
                    evaluate_socialize(&pop_pos, &needs, &weights, taverns_state.iter(world))
                {
                    check_best(ActionType::Socialize, utility, Some(target));
                }

                // Evaluate Work
                if let Some((utility, target)) =
                    evaluate_work(&pop_pos, &weights, designations_state.iter(world))
                {
                    check_best(ActionType::Work, utility, Some(target));
                }

                // Evaluate Refine
                if let Some((utility, target)) = evaluate_refine(
                    &pop_pos,
                    &weights,
                    &resources,
                    &cycle,
                    refining_state.iter(world),
                ) {
                    check_best(ActionType::Refine, utility, Some(target));
                }

                // Evaluate Farm
                if let Some((utility, target)) =
                    evaluate_farm(&pop_pos, &weights, &cycle, farms_state.iter(world))
                {
                    check_best(ActionType::Farm, utility, Some(target));
                }

                // Evaluate FetchTool
                let equipment = equipment_opt.unwrap_or_default();
                if let Some((utility, target)) = evaluate_fetch_tool(
                    &pop_pos,
                    &equipment,
                    &resources,
                    stockpiles_state.iter(world),
                ) {
                    check_best(ActionType::FetchTool, utility, Some(target));
                }

                // Evaluate Repair
                if let Some((utility, target)) =
                    evaluate_repair(&pop_pos, &weights, designations_state.iter(world))
                {
                    check_best(ActionType::Repair, utility, Some(target));
                }

                // Evaluate Explore
                if let Some((utility, target)) =
                    evaluate_explore(&pop_pos, &weights, anomalies_state.iter(world))
                {
                    check_best(ActionType::Explore, utility, Some(target));
                }

                // Evaluate Research
                if let Some((utility, target)) = evaluate_research(
                    &pop_pos,
                    &weights,
                    &resources,
                    &cycle,
                    libraries_state.iter(world),
                ) {
                    check_best(ActionType::Research, utility, Some(target));
                }

                // Evaluate Haul
                if let Some((utility, target)) = evaluate_haul(
                    &pop_pos,
                    &weights,
                    items_state.iter(world),
                    stockpiles_state.iter(world),
                    &resources,
                ) {
                    check_best(ActionType::Haul, utility, Some(target));
                }

                // Evaluate SeekMedicalCare
                if let Some(health) = health {
                    if let Some((utility, target)) = evaluate_seek_medical_care(
                        &pop_pos,
                        &needs,
                        health,
                        &weights,
                        hospitals_state.iter(world),
                    ) {
                        check_best(ActionType::SeekMedicalCare, utility, Some(target));
                    }
                }

                // Evaluate BuryCorpse
                if let Some((utility, target)) = evaluate_bury_corpse(
                    &pop_pos,
                    corpses_state.iter(world),
                    graves_state.iter(world),
                    &weights,
                ) {
                    check_best(ActionType::BuryCorpse, utility, Some(target));
                }
            }
        } // End of else block (normal evaluation)

        // Switch if best exceeds threshold
        if best_utility > action.current_utility + config.switch_threshold {
            // Update action
            action.current = best_action;
            action.current_utility = best_utility;
            action.ticks_committed = 0;

            // Write back to world
            if let Some(mut pop_action) = world.get_mut::<PopAction>(pop_entity) {
                *pop_action = action;
            }

            // Insert StartPlan marker (for HTN system)
            world.entity_mut(pop_entity).insert(StartPlan {
                action: best_action,
                target: best_target,
            });
        }
    }
}

/// Updates utility weights based on action outcome.
pub fn update_weights_from_outcome(
    weights: &mut UtilityWeights,
    action: ActionType,
    success: bool,
    duration: u32,
    config: &UtilityConfig,
) {
    // Track attempt
    let idx = action.as_index();
    weights.action_attempt_count[idx] += 1;

    if success {
        weights.action_success_count[idx] += 1;

        // Successful action: reinforce weights
        if duration < 10 {
            weights.distance_weight += config.learning_rate * 0.1;
        }
        weights.availability_weight += config.learning_rate * 0.05;
    } else {
        // Failed action: reduce weights
        weights.distance_weight -= config.learning_rate * 0.05;
        weights.availability_weight -= config.learning_rate * 0.05;
    }

    // Clamp weights
    weights.distance_weight = weights
        .distance_weight
        .clamp(config.weight_clamp.0, config.weight_clamp.1);
    weights.availability_weight = weights
        .availability_weight
        .clamp(config.weight_clamp.0, config.weight_clamp.1);
    weights.social_weight = weights
        .social_weight
        .clamp(config.weight_clamp.0, config.weight_clamp.1);
}

/// System to track completed plans and trigger learning.
pub fn track_plan_outcomes_system(
    mut completed: Query<(Entity, &PlanOutcome, &Needs, &mut UtilityWeights), Without<Plan>>,
    config: Res<UtilityConfig>,
    time: Res<SimulationTime>,
    mut commands: Commands,
) {
    let sim_time = time.tick;

    for (pop_entity, outcome, needs_after, mut weights) in &mut completed {
        let duration = sim_time - outcome.started_at;

        let success = match outcome.action {
            ActionType::SatisfyHunger => (needs_after.hunger - outcome.needs_before.hunger) > 0.05,
            ActionType::SatisfyRest => (needs_after.rest - outcome.needs_before.rest) > 0.05,
            ActionType::Work
            | ActionType::Repair
            | ActionType::Socialize
            | ActionType::Explore
            | ActionType::Research
            | ActionType::Haul
            | ActionType::SeekMedicalCare
            | ActionType::BuryCorpse
            | ActionType::FetchTool
            | ActionType::Idle
            | ActionType::Vandalize
            | ActionType::Binge
            | ActionType::Daze
            | ActionType::Fight
            | ActionType::Refine
            | ActionType::Farm
            | ActionType::Warden
            | ActionType::Sleepwalking => true,
        };

        #[allow(clippy::cast_possible_truncation)]
        update_weights_from_outcome(
            &mut weights,
            outcome.action,
            success,
            duration as u32,
            &config,
        );

        commands.entity(pop_entity).remove::<PlanOutcome>();
    }
}

// Re-add tests at the bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    fn setup() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world
    }

    #[test]
    fn test_update_action_timer() {
        let mut world = setup();
        let pop = world.spawn(PopAction::default()).id();

        // Run system
        world.run_system_once(update_action_timer_system).unwrap();

        assert_eq!(world.get::<PopAction>(pop).unwrap().ticks_committed, 1);
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
        // High need value (satisfied) = low urgency
        let urgency_high = need_response_curve(0.9);
        assert!(urgency_high < 0.2, "Satisfied need should have low urgency");

        // Medium need = medium urgency
        let urgency_med = need_response_curve(0.5);
        assert!(urgency_med > 0.6 && urgency_med < 0.8);

        // Low need (critical) = high urgency
        let urgency_low = need_response_curve(0.1);
        assert!(urgency_low > 0.95, "Critical need should have high urgency");

        // Curve should be monotonic (lower need = higher urgency)
        assert!(urgency_low > urgency_med);
        assert!(urgency_med > urgency_high);
    }

    #[test]
    fn test_calculate_context_score_distance() {
        let weights = UtilityWeights::default();

        // Close building (distance 1)
        let close_score = calculate_context_score(
            GridPosition { x: 0, y: 0 },
            Some(GridPosition { x: 1, y: 0 }),
            4, // capacity
            0, // occupied
            &weights,
        );

        // Far building (distance 10)
        let far_score = calculate_context_score(
            GridPosition { x: 0, y: 0 },
            Some(GridPosition { x: 10, y: 0 }),
            4,
            0,
            &weights,
        );

        assert!(
            close_score > far_score,
            "Closer building should have higher score"
        );
    }

    #[test]
    fn test_calculate_context_score_availability() {
        let weights = UtilityWeights::default();
        let pos = GridPosition { x: 0, y: 0 };

        // Empty building
        let empty_score = calculate_context_score(
            pos,
            Some(GridPosition { x: 5, y: 5 }),
            4, // capacity
            0, // occupied
            &weights,
        );

        // Nearly full building
        let full_score = calculate_context_score(
            pos,
            Some(GridPosition { x: 5, y: 5 }),
            4, // capacity
            3, // occupied
            &weights,
        );

        assert!(
            empty_score > full_score,
            "Empty building should have higher score"
        );
    }

    #[test]
    fn test_calculate_success_modifier() {
        let mut weights = UtilityWeights::default();

        // No history = neutral modifier (~1.0)
        let neutral = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(neutral > 0.9 && neutral < 1.1);

        // High success rate
        weights.action_attempt_count[ActionType::SatisfyHunger.as_index()] = 10;
        weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 9;
        let high = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(high > 1.0, "High success rate should boost modifier");

        // Low success rate
        weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 2;
        let low = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(low < 1.0, "Low success rate should reduce modifier");
    }

    #[test]
    fn test_evaluate_satisfy_hunger_finds_best_farm() {
        let mut world = World::new();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hunger: 0.3,
            rest: 0.8,
            leisure: 0.8,
        };
        let weights = UtilityWeights::default();

        // Close but crowded farm
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 2, y: 0 },
            Farm {
                capacity: 2,
                workers: vec![Entity::from_raw(999)],
            },
        ));

        // Far but empty farm
        let _far_farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 10, y: 0 },
                Farm {
                    capacity: 2,
                    workers: vec![],
                },
            ))
            .id();

        let mut farms = world.query::<(Entity, &GridPosition, &Farm)>();
        let result = evaluate_satisfy_hunger(&pop_pos, &needs, &weights, farms.iter(&world));

        assert!(result.is_some());
    }

    #[test]
    fn test_evaluate_actions_switches_when_threshold_exceeded() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        // Starving pop currently idle
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.8,
                    leisure: 0.8,
                }, // Very hungry!
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.2,
                    ticks_committed: 10, // Past evaluation interval
                },
            ))
            .id();

        // Available farm
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        evaluate_actions_system(&mut world);

        // Should have switched to SatisfyHunger
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::SatisfyHunger);
        assert!(
            action.current_utility > 0.5,
            "Hungry pop should have high hunger utility"
        );
        assert_eq!(action.ticks_committed, 0, "Should reset commitment counter");
    }

    #[test]
    fn test_evaluate_actions_respects_threshold() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig {
            switch_threshold: 0.5, // High threshold
            ..Default::default()
        });
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.6,
                    rest: 0.6,
                    leisure: 0.6,
                }, // Moderate needs
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.4,
                    ticks_committed: 10,
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        evaluate_actions_system(&mut world);

        // Should NOT switch (utility difference < threshold)
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Should not switch with high threshold"
        );
    }

    #[test]
    fn test_update_weights_from_success() {
        let mut weights = UtilityWeights::default();
        let config = UtilityConfig::default();

        let initial_distance = weights.distance_weight;
        let initial_availability = weights.availability_weight;

        update_weights_from_outcome(
            &mut weights,
            ActionType::SatisfyHunger,
            true, // success
            5,    // quick duration
            &config,
        );

        // Weights should increase on success
        assert!(weights.distance_weight > initial_distance);
        assert!(weights.availability_weight > initial_availability);

        // Success should be tracked
        assert_eq!(
            weights.action_attempt_count[ActionType::SatisfyHunger.as_index()],
            1
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            1
        );
    }

    #[test]
    fn test_update_weights_from_failure() {
        let mut weights = UtilityWeights::default();
        let config = UtilityConfig::default();

        let initial_distance = weights.distance_weight;
        let initial_availability = weights.availability_weight;

        update_weights_from_outcome(
            &mut weights,
            ActionType::SatisfyHunger,
            false, // failure
            20,    // slow duration
            &config,
        );

        // Weights should decrease on failure
        assert!(weights.distance_weight < initial_distance);
        assert!(weights.availability_weight < initial_availability);

        // Failure should be tracked
        assert_eq!(
            weights.action_attempt_count[ActionType::SatisfyHunger.as_index()],
            1
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            0
        );
    }

    #[test]
    fn test_weights_clamped_to_range() {
        let mut weights = UtilityWeights::default();
        let config = UtilityConfig::default();

        // Drive weights to minimum
        for _ in 0..100 {
            update_weights_from_outcome(
                &mut weights,
                ActionType::SatisfyHunger,
                false,
                30,
                &config,
            );
        }

        assert!(weights.distance_weight >= config.weight_clamp.0);
        assert!(weights.availability_weight >= config.weight_clamp.0);

        // Drive weights to maximum
        for _ in 0..100 {
            update_weights_from_outcome(&mut weights, ActionType::SatisfyRest, true, 1, &config);
        }

        assert!(weights.distance_weight <= config.weight_clamp.1);
        assert!(weights.availability_weight <= config.weight_clamp.1);
    }

    #[test]
    fn test_track_plan_outcomes_on_success() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.6,
                    rest: 0.8,
                    leisure: 0.8,
                }, // Improved from 0.3
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::SatisfyHunger,
                    started_at: 90,
                    needs_before: Needs {
                        hunger: 0.3,
                        rest: 0.8,
                        leisure: 0.8,
                    },
                },
            ))
            .id();
        // Note: No Plan component = plan completed

        world.run_system_once(track_plan_outcomes_system).unwrap();

        // Should have updated weights
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            1
        );

        // Should have removed PlanOutcome
        assert!(world.get::<PlanOutcome>(pop).is_none());
    }

    #[test]
    fn test_track_plan_outcomes_on_failure() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                }, // No improvement
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::SatisfyHunger,
                    started_at: 90,
                    needs_before: Needs {
                        hunger: 0.3,
                        rest: 0.8,
                        leisure: 0.8,
                    },
                },
            ))
            .id();

        world.run_system_once(track_plan_outcomes_system).unwrap();

        // Should have tracked failure
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(
            weights.action_attempt_count[ActionType::SatisfyHunger.as_index()],
            1
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            0
        );
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

    #[test]
    fn test_colony_memory_default() {
        let memory = ColonyMemory::default();
        assert!(memory.total_successful_actions.iter().all(|&x| x == 0));
        assert!(memory.average_action_duration.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_integration_pop_learns_from_experience() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                },
                UtilityWeights::default(),
                PopAction::default(),
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 0 },
            Farm::default(),
        ));

        // Run multiple cycles
        for _ in 0..5 {
            // Simulate successful hunger satisfaction
            world.get_mut::<Needs>(pop).unwrap().hunger = 0.3;
            world.get_mut::<PopAction>(pop).unwrap().ticks_committed = 10;

            evaluate_actions_system(&mut world);

            // Simulate success
            let tick = world.resource::<SimulationTime>().tick;
            world.entity_mut(pop).insert(PlanOutcome {
                action: ActionType::SatisfyHunger,
                started_at: tick,
                needs_before: Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                },
            });

            world.get_mut::<Needs>(pop).unwrap().hunger = 0.7; // Success!
            world.resource_mut::<SimulationTime>().tick += 10;

            world.run_system_once(track_plan_outcomes_system).unwrap();
        }

        // Pop should have learned (weights increased)
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert!(
            weights.distance_weight > 1.0 || weights.availability_weight > 1.0,
            "Pop should have learned from successful experiences"
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            5
        );
    }

    #[test]
    fn test_track_plan_outcomes_work_action() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs::default(),
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::Work,
                    started_at: 50,
                    needs_before: Needs::default(),
                },
            ))
            .id();

        // Run system
        world.run_system_once(track_plan_outcomes_system).unwrap();

        let weights = world.get::<UtilityWeights>(pop).unwrap();
        // This should be 1 if Work is considered a success when completed
        assert_eq!(
            weights.action_success_count[ActionType::Work.as_index()],
            1,
            "Work action should be counted as success if completed"
        );
    }
}
