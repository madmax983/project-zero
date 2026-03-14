use bevy_ecs::prelude::*;
use crate::layer1::social::SocialInteractionEvent;
use crate::layer1::utility_types::{ActionType, UtilityAIBuffer, PopAction};
use crate::layer1::utility_eval_types::PopDecider;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct MemeticInfection {
    pub obsession_type: ObsessionType,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObsessionType {
    StackChairs,
    DigHoles,
}

#[derive(Component)]
pub struct Quarantined;

pub fn process_memetic_transmission_system(
    mut events: EventReader<SocialInteractionEvent>,
    mut commands: Commands,
    infected_query: Query<&MemeticInfection, Without<Quarantined>>,
    healthy_query: Query<Entity, (With<crate::layer1::pop::Pop>, Without<MemeticInfection>, Without<Quarantined>)>,
) {
    for event in events.read() {
        if let Ok(infection) = infected_query.get(event.initiator) {
            if healthy_query.get(event.target).is_ok() {
                commands.entity(event.target).insert(infection.clone());
            }
        }
    }
}

pub fn evaluate_obsession_utility_system(
    mut query: Query<(&MemeticInfection, &mut UtilityAIBuffer)>,
) {
    for (infection, mut buffer) in query.iter_mut() {
        // High utility to override other needs
        let decision = PopDecider::consider(ActionType::MemeticObsession, 10.0, None);
        // Wait, UtilityAIBuffer has no `add_action`. It holds an array of options or we just set the action?
        // Let's look up how utility is stored.
    }
}
