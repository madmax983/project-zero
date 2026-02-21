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

pub use self::types::*;
pub use self::utils::*;
pub use self::combat::*;
pub use self::movement::*;
pub use self::arrival::*;
pub use self::work::*;

pub mod types;
pub mod utils;
pub mod combat;
pub mod movement;
pub mod arrival;
pub mod work;

#[cfg(test)]
mod tests;
