//! The Cognitive Engine of the Colony.
//!
//! The `mind` module serves as the central nervous system for every Pop in SCALE.
//! Rather than scripting explicit behaviors (e.g., "if hungry, go eat"), this
//! system uses a **Utility AI** architecture to score every possible action against
//! the Pop's current needs, environment, and personality traits. The action with
//! the highest utility wins.
//!
//! # Core Philosophy: Emergence over Scripting
//!
//! By evaluating actions mathematically, Pops exhibit complex, emergent behaviors:
//! - A starving Pop will walk across the map for food, ignoring their job.
//! - A slightly tired Pop will finish their shift before seeking a bed, unless that bed is right next to them.
//! - A Pop with a `distance_weight` of 2.0 (lazy) will rather eat raw food nearby than walk to a tavern for a cooked meal.
//!
//! # The "Gather-Think-Act" Cycle
//!
//! Because Bevy ECS prevents mutable access to the world from multiple threads,
//! the `mind` module operates in three distinct phases to achieve high parallelism:
//!
//! 1. **Gather**: We extract all necessary read-only data (Pops, Needs, Buildings, Items)
//!    from the world and place it into a massive `UtilityAIBuffer`.
//! 2. **Think ([`utility_ai`])**: Thousands of Pops evaluate their options simultaneously across all available CPU cores using `bevy_tasks::ComputeTaskPool`.
//!    They read from the buffer and output their chosen action.
//! 3. **Act**: The results are written back to the [`PopAction`] components in the ECS world.
//!
//! # Submodules
//!
//! - [`utility_ai`]: The core evaluation logic and "Conscience" of the Pop.
//! - [`utility_types`]: The vocabulary of the AI, defining the `ActionType` menu and scoring math.
//! - [`utility_ai_population`]: Helpers for gathering entities into the evaluation buffer.
//! - `utility_eval_types` (internal): The data backbone, defining how we query and store context for evaluation.
//!
//! # Examples
//!
//! ```
//! use scale::layer1::mind::utility_types::{ActionType, UtilityWeights, calculate_context_score};
//! use scale::layer1::map::GridPosition;
//!
//! // Example of how Utility AI scores an option based on distance and crowding
//! let pop_pos = GridPosition { x: 0, y: 0 };
//! let target_pos = GridPosition { x: 10, y: 0 }; // 10 tiles away
//! let weights = UtilityWeights::default();
//!
//! // Calculate how appealing this target is
//! let score = calculate_context_score(
//!     pop_pos,
//!     Some(target_pos),
//!     10, // Max capacity of the building
//!     5,  // Currently 5/10 occupied
//!     &weights
//! );
//!
//! // The score will be < 1.0 due to distance and crowding penalties
//! assert!(score < 1.0);
//! ```

/// Emergent utility AI system.
pub mod fugue;
pub mod utility_ai;
#[cfg(test)]
/// Tests for utility AI hierarchy logic.
pub mod utility_ai_hierarchy_tests;
/// Utility AI population helpers.
pub mod utility_ai_population;
#[cfg(test)]
/// Tests for utility AI work logic.
pub mod utility_ai_work_tests;
/// Shared types for utility AI evaluation (`PopEvalData`, Proxies).
pub(crate) mod utility_eval_types;
/// Shared types for utility AI state (`ActionType`, `UtilityWeights`).
pub mod utility_types;

pub use fugue::*;
pub use utility_ai::*;
pub use utility_types::*;
pub mod sleep_debt;
pub mod temporal_fugue;
pub use temporal_fugue::*;
