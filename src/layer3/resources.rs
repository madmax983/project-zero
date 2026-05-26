//! Layer 3 Resources
//!
//! Defines the global economic resources used at the interstellar level,
//! such as `EmpireCredits`, which drive galactic trade and diplomacy.

use bevy_ecs::prelude::*;

/// Tracks the amount of Empire Credits available in the global Layer 3 economy.
#[derive(Resource, Default, Debug, Clone)]
pub struct EmpireCredits(pub f32);
