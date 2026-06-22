//! "Hit Stop" Simulation Freeze.
//!
//! This module provides the `GlobalHitStop` resource, a purely aesthetic "Juice" component.
//! Hit Stop is a common game design technique where the entire simulation pauses for a few
//! frames to emphasize a heavy impact, explosion, or dramatic event.
//!
//! While the logic systems (AI, pathfinding, movement) skip their execution during a Hit Stop,
//! rendering and purely visual effects (like screen shake or particles) continue, creating a
//! sense of immense force and weight.
//!
//! # Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::physics::hit_stop::GlobalHitStop;
//!
//! let mut world = World::new();
//! world.insert_resource(GlobalHitStop::default());
//!
//! // A massive explosion occurs! Trigger a 5-tick freeze.
//! let mut hit_stop = world.resource_mut::<GlobalHitStop>();
//! hit_stop.trigger(5);
//!
//! assert_eq!(hit_stop.ticks, 5);
//!
//! // Note: If another smaller event happens, it does not reduce the current freeze.
//! hit_stop.trigger(2);
//! assert_eq!(hit_stop.ticks, 5);
//! ```
use bevy_ecs::prelude::*;

/// Global resource to pause the simulation for "Juice" (Hit Stop).
///
/// When `ticks` > 0, the simulation loop (physics, AI, logic) is skipped,
/// but rendering and visual effects (Screen Shake) continue.
#[derive(Resource, Default, Debug)]
pub struct GlobalHitStop {
    /// Number of ticks to freeze the simulation.
    pub ticks: u32,
}

impl GlobalHitStop {
    /// Trigger a freeze for the given duration.
    /// Overwrites existing freeze if the new one is longer.
    pub fn trigger(&mut self, duration: u32) {
        if duration > self.ticks {
            self.ticks = duration;
        }
    }
}

/// Component for "Hit Stop" (Freeze Frame) effect.
/// Pauses the entity for a few ticks to emphasize impact.
/// Ludwig: "This adds crunch to the combat!"
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct HitStop {
    /// Ticks remaining until the entity can act again.
    pub ticks_remaining: u32,
}

/// System to process Hit Stop durations.
/// Decrements the counter and removes the component when it expires.
pub fn hit_stop_system(mut commands: Commands, mut query: Query<(Entity, &mut HitStop)>) {
    for (entity, mut hit_stop) in &mut query {
        if hit_stop.ticks_remaining > 0 {
            hit_stop.ticks_remaining -= 1;
        } else {
            commands.entity(entity).remove::<HitStop>();
        }
    }
}
