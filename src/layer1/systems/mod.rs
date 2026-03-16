//! System registration and sets (Facade).
//!
//! This module coordinates the registration of all Layer 1 systems.
//! To avoid a single "God File", the systems are split into sub-modules based on their
//! execution phase:
//!
//! * [`cleanup`](crate::layer1::systems::cleanup): Event cleanup and input handling.
//! * [`execution`](crate::layer1::systems::execution): Movement, work, and direct interactions.
//! * [`economy`](crate::layer1::systems::economy): Production, resources, and passive ticking.
//! * [`environment`](crate::layer1::systems::environment): Fire, weather, and decay.
//! * [`consumption`](crate::layer1::systems::consumption): Needs decay, spoilage, and death.
//! * [`observation`](crate::layer1::systems::observation): History, social, and dreams.

use bevy_ecs::prelude::*;

pub mod cleanup;
pub mod consumption;
pub mod economy;
pub mod environment;
pub mod execution;
pub mod observation;

/// Helper system to update event buffers (clear old events).
pub fn update_event_buffer<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}

/// System sets for Layer 1 simulation phases.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Layer1SystemSet {
    /// Cleanup of previous frame's events.
    EventCleanup,
    /// Execution of plans (movement, work, etc.).
    Execution,
    /// Economic production and resource management.
    Economy,
    /// Environmental effects (fire, weather).
    Environment,
    /// Consumption and decay (needs, spoilage).
    Consumption,
    /// Observation and social systems (history, dreams).
    Observation,
}

/// Registers all Layer 1 systems into the provided schedule.
///
/// This function groups systems into ordered `SystemSet`s to enforce
/// execution order and logical grouping.
#[allow(clippy::too_many_lines)]
pub fn register_layer1_systems(schedule: &mut Schedule) {
    // Configure Sets
    schedule.configure_sets((
        Layer1SystemSet::EventCleanup,
        Layer1SystemSet::Execution.after(Layer1SystemSet::EventCleanup),
        Layer1SystemSet::Economy.after(Layer1SystemSet::Execution),
        Layer1SystemSet::Environment.after(Layer1SystemSet::Economy),
        Layer1SystemSet::Consumption.after(Layer1SystemSet::Economy), // Parallel with Environment
        Layer1SystemSet::Observation.after(Layer1SystemSet::Consumption),
    ));

    cleanup::register(schedule);
    execution::register(schedule);
    economy::register(schedule);
    environment::register(schedule);
    consumption::register(schedule);
    observation::register(schedule);
}
