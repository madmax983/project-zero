//! # Orbital and Strategic Events
//!
//! This module defines the core, foundational events used across Layer 2 (Orbital/Strategic layer)
//! systems. These events act as the primary communication mechanism between different strategic
//! sub-systems, such as fleet management, planetary defense, and stealth mechanics.
//!
//! Using events instead of direct state polling allows decoupling between the entity that triggers
//! a strategic action (like launching a ship) and the entities that react to it (like tracking
//! logistics, adjusting visibility, or triggering enemy responses).

use bevy_ecs::prelude::*;

/// Event triggered when a launch occurs from a planetary body.
///
/// This event is used to notify the strategic layer that a ship or payload has attempted
/// to leave the planet's gravity well. Systems tracking orbital traffic, fuel consumption,
/// or launch pad availability should listen to this event.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer2::events::LaunchEvent;
///
/// let mut world = World::new();
/// world.insert_resource(Events::<LaunchEvent>::default());
///
/// let planet_entity = world.spawn_empty().id();
///
/// world.send_event(LaunchEvent {
///     planet: planet_entity,
///     success: true,
/// });
/// ```
#[derive(Event, Debug, Clone)]
pub struct LaunchEvent {
    /// The planet entity the launch originated from.
    pub planet: Entity,
    /// Whether the launch successfully breached the atmosphere.
    pub success: bool,
}

/// Event triggered when a ship is destroyed in orbit or deep space.
///
/// Systems managing debris generation, fleet morale, and casualty tracking should
/// listen to this event. It abstracts the complexity of combat or environmental
/// damage down to a simple notification of asset loss.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer2::events::ShipDestroyedEvent;
///
/// let mut world = World::new();
/// world.insert_resource(Events::<ShipDestroyedEvent>::default());
///
/// let planet_entity = world.spawn_empty().id();
///
/// world.send_event(ShipDestroyedEvent {
///     planet: planet_entity,
///     ship_class: "Frigate".to_string(),
/// });
/// ```
#[derive(Event, Debug, Clone)]
pub struct ShipDestroyedEvent {
    /// The nearest planet or the planet the ship belonged to when destroyed.
    pub planet: Entity,
    /// The classification of the destroyed ship (e.g., "Frigate", "Colony Ship").
    pub ship_class: String,
    /// Whether the destroyed ship was a veteran.
    pub is_veteran: bool,
}

/// Event triggered when the colony's thermal or electromagnetic signature is detected by hostile forces.
///
/// This is typically fired by stealth or visibility systems when the cumulative signature
/// exceeds a critical threshold, prompting an escalation in threat level or immediate reprisal.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer2::events::DetectionEvent;
///
/// let mut world = World::new();
/// world.insert_resource(Events::<DetectionEvent>::default());
///
/// world.send_event(DetectionEvent);
/// ```
#[derive(Event, Debug, Clone, Default)]
pub struct DetectionEvent;
