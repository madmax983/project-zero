//! Diplomacy and Faction Relations
//!
//! This module manages interactions between the colony and external factions,
//! entities, or neighboring groups. It handles treaties, diplomatic wards,
//! and the shifting allegiances within the simulation.
//!
//! # Mechanics
//! - **Wards:** Logic for managing diplomatic protections, alliances, or specific
//!   zones of control influenced by external relations.

pub mod wards;
pub use wards::*;
