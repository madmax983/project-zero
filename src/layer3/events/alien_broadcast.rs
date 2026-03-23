use bevy_ecs::prelude::*;
use rand::Rng;

/// Event triggered when a hostile alien civilization broadcasts a catchy audio signal.
/// This acts as a vector for `MemeticInfection::ParasiticBroadcast`.
#[derive(Event, Debug, Clone, Default)]
pub struct AlienBroadcastEvent;

/// Randomly triggers an `AlienBroadcastEvent` with a low chance per tick.
pub fn trigger_alien_broadcast_system(mut events: EventWriter<AlienBroadcastEvent>) {
    let mut rng = rand::thread_rng();
    // 0.01% chance per tick to receive the broadcast.
    if rng.gen_bool(0.0001) {
        events.send(AlienBroadcastEvent);
    }
}
