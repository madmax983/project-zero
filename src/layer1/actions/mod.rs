//! # The Action System
//!
//! The "Action System" is the interface between the **[Utility AI](crate::layer1::utility_ai)** brain
//! and the **Execution** systems. It defines *what* a Pop can do, *how* it decides to do it,
//! and *who* or *what* it interacts with.
//!
//! ## 1. The Decision Lifecycle
//!
//! Every few ticks, a Pop evaluates its options:
//!
//! 1.  **Candidate Gathering**: The [`crate::layer1::utility_ai::evaluate_actions_system`] collects
//!     all potential targets (Farms, Mines, Stockpiles) into a [`crate::layer1::utility_eval_types::UtilityAIBuffer`].
//! 2.  **Scoring**: The system calls specific `evaluate_*` functions (e.g., [`crate::layer1::actions::work::evaluate_work`])
//!     for each action type.
//!     *   Each function returns an `Option<(f32, Entity)>`: The best score (0.0-1.0+) and the target entity.
//!     *   Scores are based on **Need Urgency**, **Distance**, and **Personality Weights**.
//! 3.  **Selection**: The highest scoring action becomes the `PopAction::current`.
//! 4.  **Assignment**: The pop is assigned to the target entity via the [`AssignedTo`] component.
//!
//! ## 2. The Contract: `evaluate_*`
//!
//! All evaluation functions follow a strict pattern:
//!
//! ```rust,ignore
//! fn evaluate_example(
//!     pop_pos: GridPosition,
//!     weights: &UtilityWeights,
//!     candidates: &[CandidateProxy]
//! ) -> Option<(f32, Entity)>
//! ```
//!
//! *   **Input**: Pop state and a list of lightweight candidate proxies (not full entities).
//! *   **Process**: Iterate candidates, calculate [`crate::layer1::utility_types::calculate_context_score`], multiply by [`crate::layer1::utility_types::calculate_success_modifier`].
//! *   **Output**: The single best target and its score.
//!
//! ## 3. Execution
//!
//! Once an action is selected, specific systems in `src/layer1/execution.rs` (or action-specific modules)
//! handle the actual behavior (moving, working, modifying state).
//!
//! ## Example: Defining a New Action
//!
//! To add a new action (e.g., `Pray`):
//! 1.  Add `ActionType::Pray` to [`crate::layer1::utility_types::ActionType`].
//! 2.  Create `src/layer1/actions/pray.rs` with an `evaluate_pray` function.
//! 3.  Register the evaluation in [`crate::layer1::utility_ai::evaluate_single_pop`].
//! 4.  Implement the execution logic (e.g., `pray_execution_system`).
//!
//! # Action Registry
//!
//! While most actions are defined in this module, some domain-specific actions live in their respective modules:
//!
//! | Action Type | Evaluation Logic | Execution Logic |
//! |:---|:---|:---|
//! | **Warden** | [`crate::layer1::justice::evaluate_warden_action`] | [`crate::layer1::justice::warden_execution_system`] |
//! | **Tame** | [`crate::layer1::husbandry::evaluate_tame`] | [`crate::layer1::husbandry::tame_execution_system`] |
//! | **Sleepwalking** | [`crate::layer1::actions::mental_break::evaluate_mental_break`] | [`crate::layer1::sleepwalking::check_sleepwalking_start_system`] |
//! | **Vandalize** | [`crate::layer1::actions::mental_break::evaluate_mental_break`] | [`crate::layer1::execution::vandalize_execution_system`] |
//! | **Surgery** | *Passive / Assigned* | [`crate::layer1::cybernetics::surgery_system`] |

pub use crate::layer1::utility_types::AssignmentType;
use bevy_ecs::prelude::*;

/// Hunger satisfaction action logic.
pub mod hunger;
/// Rest satisfaction action logic.
pub mod rest;

/// Explore action logic.
pub mod explore;
/// Fetch clothing action logic.
pub mod fetch_clothing;
/// Fetch tool action logic.
pub mod fetch_tool;
/// Haul action logic.
pub mod haul;
/// Repair action logic.
pub mod repair;
/// Research action logic.
pub mod research;
/// Work action logic.
pub mod work;

/// Refine action logic.
pub mod refine;

/// Admin action logic.
pub mod admin;
/// Farm action logic.
pub mod farm;

/// Fight action logic.
pub mod fight;
/// Funeral action logic.
pub mod funeral;
/// Medical action logic.
pub mod medical;
/// Social action logic.
pub mod social;

/// Mental break action logic.
pub mod mental_break;

#[cfg(test)]
mod work_building_tests;

/// Component tracking what a pop is assigned to.
///
/// This component links a Pop to a specific entity (Building, Item, etc.) for the duration
/// of their action. It is used by systems to:
/// *   Reserve capacity (e.g., "This bed is taken").
/// *   Visualize relationships (e.g., "Working at Farm").
/// *   Prevent multiple pops from claiming the same resource (in some cases).
///
/// # Examples
///
/// ```
/// use scale::layer1::actions::{AssignedTo, AssignmentType};
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let farm_entity = world.spawn_empty().id();
/// let pop_entity = world.spawn(AssignedTo {
///     entity: farm_entity,
///     assignment_type: AssignmentType::FarmWorker,
/// }).id();
/// ```
#[derive(Component, Debug)]
pub struct AssignedTo {
    /// The entity the pop is assigned to.
    pub entity: Entity,
    /// The type of assignment.
    pub assignment_type: AssignmentType,
}
