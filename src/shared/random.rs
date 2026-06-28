//! Random number generation utilities.
//!
//! This module provides deterministic randomness states, allowing the game
//! to reliably regenerate the same galaxy and procedural histories given the
//! same initial seed.

use bevy_ecs::prelude::Resource;

/// The seed used for procedural generation.
///
/// This ensures that the galaxy generation is deterministic.
///
/// # Examples
///
/// ```
/// use scale::shared::random::WorldSeed;
///
/// let seed = WorldSeed(42);
/// assert_eq!(seed.0, 42);
/// ```
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);

/// Global RNG resource for deterministic simulation
#[derive(Resource)]
pub struct GlobalRng(pub rand::rngs::StdRng);
