/// Emergent utility AI system.
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

pub use utility_ai::*;
pub use utility_types::*;
