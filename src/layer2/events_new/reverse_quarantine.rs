//! # The Reverse Quarantine Event
//!
//! This module simulates the "Reverse Quarantine" narrative event: a desperate refugee
//! fleet arrives at the colony, begging for asylum. However, accepting them risks
//! introducing an unknown plague, while rejecting them causes immense guilt among
//! the colonists and scattered orbital debris as the fleet is forcibly repelled.
//!
//! This serves as a moral and strategic dilemma test-bed for the events system.

use crate::layer1::environment::events::DebrisFallEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::MoodModifier;
use crate::layer1::morale::Morale;
use bevy_ecs::prelude::*;

/// The strategic decision made in response to the refugee fleet's arrival.
///
/// This choice dictates the subsequent cascading effects on the colony's morale,
/// physical safety (via orbital debris), and potential disease outbreaks.
/// Event representing the arrival of a refugee fleet seeking asylum.
///
/// This event is used to trigger the "Reverse Quarantine" scenario. It holds
/// the fleet's parameters and optionally the decision made by the player or AI
/// regarding their fate. If `decision` is `None`, the event is still pending a choice.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer1::map::GridPosition;
/// use scale::layer2::events_new::reverse_quarantine::{RefugeeFleetEvent};
///
/// let mut world = World::new();
/// world.insert_resource(Events::<RefugeeFleetEvent>::default());
///
/// world.send_event(RefugeeFleetEvent {
///     fleet_size: 3,
///     is_accepted: Some(false),
///     target_location: GridPosition { x: 10, y: 20 },
/// });
/// ```
#[derive(Event, Clone, Copy)]
pub struct RefugeeFleetEvent {
    /// The number of ships in the refugee fleet.
    pub fleet_size: u32,
    /// The decision on how to handle the fleet. `None` if unresolved.
    pub is_accepted: Option<bool>,
    /// The coordinates above the colony where the fleet is holding orbit.
    pub target_location: GridPosition,
}

/// Processes the resolution of a refugee fleet encounter.
///
/// This system listens for resolved [`RefugeeFleetEvent`]s (where `decision` is `Some`).
///
/// ## Consequences
///
/// *   **[``false``]**: Iterates through all ships based on `fleet_size` and
///     dispatches a [`DebrisFallEvent`] for each, simulating the forceful repulsion
///     of the fleet. It also applies a negative [`MoodModifier`] to all Pops in the
///     colony, simulating the collective guilt of turning away the desperate.
/// *   **[``true``]**: (Currently a placeholder for MVP). Intended to
///     introduce new, potentially infected Pops to the colony.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer1::environment::events::DebrisFallEvent;
/// use scale::layer1::map::GridPosition;
/// use scale::layer1::morale::Morale;
/// use scale::layer2::events_new::reverse_quarantine::{process_refugee_decisions_system, RefugeeFleetEvent};
///
/// let mut world = World::new();
/// world.insert_resource(Events::<RefugeeFleetEvent>::default());
/// world.insert_resource(Events::<DebrisFallEvent>::default());
///
/// // Spawn a colonist to receive guilt
/// let colonist = world.spawn(Morale { value: 1.0, ..Default::default() }).id();
///
/// world.send_event(RefugeeFleetEvent {
///     fleet_size: 2,
///     is_accepted: Some(false),
///     target_location: GridPosition { x: 0, y: 0 },
/// });
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(process_refugee_decisions_system);
/// schedule.run(&mut world);
///
/// // Debris falls from repelling 2 ships
/// let debris_events = world.resource::<Events<DebrisFallEvent>>();
/// assert_eq!(debris_events.get_cursor().len(debris_events), 2);
///
/// // Colonist suffers guilt
/// let morale = world.get::<Morale>(colonist).unwrap();
/// assert_eq!(morale.modifiers[0].value, -0.25);
/// ```
pub fn process_refugee_decisions_system(
    mut events: EventReader<RefugeeFleetEvent>,
    mut debris_events: EventWriter<DebrisFallEvent>,
    mut pops: Query<&mut Morale>,
) {
    for event in events.read() {
        if let Some(false) = event.is_accepted {
            // Repelling the fleet causes massive debris
            for i in 0..event.fleet_size {
                debris_events.send(DebrisFallEvent {
                    location: GridPosition {
                        x: event.target_location.x + (i as i32), // Simulate scatter
                        y: event.target_location.y,
                    },
                    severity: 50.0,
                });
            }

            // Apply guilt to all pops
            for mut morale in pops.iter_mut() {
                morale.add_modifier(MoodModifier {
                    value: -0.25, // -25% mood / stress spike
                    duration: 100,
                    label: "Guilt (Refugees Rejected)".to_string(),
                });
            }
        }
        // `true` logic omitted for MVP
    }
}
