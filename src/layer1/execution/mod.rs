//! Execution layer bridging utility AI decisions to actual pop actions.
//!
//! This module handles:
//! - Movement toward targets
//! - Assigning pops to farms/housing
//! - Executing work at designations (mining, chopping)
//!
//! # System Flow
//!
//! ```text
//! evaluate_actions_system (existing)
//!     ↓ inserts StartPlan
//! cleanup_previous_assignment_system
//!     ↓ removes pop from old farm/housing
//! process_start_plan_system
//!     ↓ converts StartPlan → MovementTarget
//! movement_system
//!     ↓ moves pop 1 tile, inserts AtTarget on arrival
//! arrival_handler_system
//!     ↓ assigns to farm/housing
//! work_execution_system
//!     ↓ calls mine_rock/chop_tree
//! ```

#![allow(clippy::collapsible_if, clippy::cast_precision_loss)]

/// Arrival handling logic.
pub mod arrival;
/// Combat execution logic.
pub mod combat;
/// Shared components for execution.
pub mod components;
/// Demolition execution logic.
pub mod demolish;
/// General work execution logic.
pub mod general_work;
/// Mining and chopping execution logic.
pub mod mining;
/// Movement system.
pub mod movement;
/// Vandalism execution logic.
pub mod vandalism;
/// Work efficiency calculations (Spec 218).
pub mod efficiency;

#[cfg(test)]
mod tests;

pub use arrival::*;
pub use efficiency::*;
pub use combat::*;
pub use components::*;
pub use demolish::*;
pub use general_work::*;
pub use mining::*;
pub use movement::*;
pub use vandalism::*;
