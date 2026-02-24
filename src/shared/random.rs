use bevy_ecs::prelude::Resource;

/// The seed used for procedural generation.
///
/// This ensures that the galaxy generation is deterministic.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);
