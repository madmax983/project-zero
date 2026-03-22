use crate::layer1::environment::events::DebrisFallEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::MoodModifier;
use crate::layer1::morale::Morale;
use bevy_ecs::prelude::*;

#[derive(Clone, Copy)]
pub enum Decision {
    Accept,
    Reject,
}

#[derive(Event, Clone, Copy)]
pub struct RefugeeFleetEvent {
    pub fleet_size: u32,
    pub decision: Option<Decision>,
    pub target_location: GridPosition,
}

pub fn process_refugee_decisions_system(
    mut events: EventReader<RefugeeFleetEvent>,
    mut debris_events: EventWriter<DebrisFallEvent>,
    mut pops: Query<&mut Morale>,
) {
    for event in events.read() {
        if let Some(Decision::Reject) = event.decision {
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
        // Decision::Accept logic omitted for MVP
    }
}
