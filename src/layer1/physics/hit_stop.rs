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
