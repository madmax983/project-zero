//! # Emergent Event Chains
//!
//! This module handles complex, multi-stage events that introduce narrative-driven
//! dilemmas to the colony. Unlike instantaneous events (such as a ship launch or
//! sudden detection), the events in this module often require an explicit player
//! or AI policy decision and unfold over time.
//!
//! ## Overview
//!
//! Systems within this module typically:
//! 1. Listen for a trigger condition.
//! 2. Dispatch a prompt for a strategic decision.
//! 3. Process the chosen decision and distribute the subsequent consequences
//!    (such as morale shifts, resource changes, or combat encounters) to the
//!    rest of the simulation.

pub mod reverse_quarantine;
pub mod reverse_quarantine_tests;
