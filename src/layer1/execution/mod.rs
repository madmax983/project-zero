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

pub mod arrival;
pub mod combat;
pub mod components;
pub mod demolish;
pub mod mining;
pub mod movement;
pub mod vandalism;
pub mod general_work;

#[cfg(test)]
mod tests;

pub use arrival::*;
pub use combat::*;
pub use components::*;
pub use demolish::*;
pub use mining::*;
pub use movement::*;
pub use vandalism::*;
pub use general_work::*;
