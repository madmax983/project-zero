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
