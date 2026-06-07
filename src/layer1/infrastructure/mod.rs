//! Infrastructure and Colony Support Systems
//!
//! This module defines the core infrastructure components that support the colony's
//! daily operations. This includes transit networks, roads, and other essential
//! connective systems that facilitate the movement of pops and resources.
//!
//! # Mechanics
//! - **Transit:** Systems for managing roads, tubes, and other pathways that affect
//!   the speed and efficiency of pop movement across the colony.

pub mod transit;
pub use transit::*;

pub mod ancient;
pub use ancient::*;

pub mod subconscious_grid;
pub use subconscious_grid::*;
