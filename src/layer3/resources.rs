use bevy_ecs::prelude::*;

/// Tracks the amount of Empire Credits available in the global Layer 3 economy.
#[derive(Resource, Default, Debug, Clone)]
pub struct EmpireCredits(pub f32);
