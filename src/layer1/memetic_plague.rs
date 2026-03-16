use bevy_ecs::prelude::*;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use crate::layer1::social::SocialInteractionEvent;

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

#[allow(clippy::type_complexity)]
pub fn process_memetic_transmission_system(
    mut events: EventReader<SocialInteractionEvent>,
    mut commands: Commands,
    infected_query: Query<&MemeticInfection, Without<Quarantined>>,
    healthy_query: Query<Entity, (With<crate::layer1::pop::Pop>, Without<MemeticInfection>, Without<Quarantined>)>,
) {
    for event in events.read() {
        if let Ok(infection) = infected_query.get(event.initiator) {
            if healthy_query.get(event.target).is_ok() {
                // Only infect target if they are close enough or if infection chance hits. The spec says spread via social interactions.
                // Infect the target
                commands.entity(event.target).insert(infection.clone());
            }
        }
    }
}

// In the current architecture, evaluate_single_pop calls individual evaluators.
// So we can just define evaluate_obsession_utility here.
pub fn evaluate_obsession_utility(
    data: &PopEvalData,
    _buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if data.is_memetic_infected {
        // High utility to override other needs
        return Some((ActionType::MemeticObsession, 10.0, None));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::SocialInteractionEvent;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<SocialInteractionEvent>>();
        world
    }

    #[test]
    fn test_memetic_infection_spread_via_social_interaction() {
        let mut world = setup_world();

        let infected_pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::StackChairs,
                intensity: 1.0,
            }
        )).id();

        let healthy_pop = world.spawn(Pop).id();

        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        world.run_system_once(process_memetic_transmission_system).unwrap();

        assert!(world.entity(healthy_pop).contains::<MemeticInfection>());
    }

    #[test]
    fn test_infected_pop_prioritizes_obsession_task() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },
        )).id();

        let buffer = UtilityAIBuffer::default();
        let mut data = PopEvalData::test_instance();
        data.entity = pop;
        data.is_memetic_infected = true;

        let result = evaluate_obsession_utility(&data, &buffer);
        assert!(result.is_some());
        let (action, utility, _target) = result.unwrap();
        assert_eq!(action, ActionType::MemeticObsession);
        assert!(utility > 1.0); // High utility
    }

    #[test]
    fn test_quarantine_prevents_transmission() {
        let mut world = setup_world();

        let infected_pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },
            Quarantined,
        )).id();

        let healthy_pop = world.spawn(Pop).id();

        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        world.run_system_once(process_memetic_transmission_system).unwrap();

        assert!(!world.entity(healthy_pop).contains::<MemeticInfection>());
    }
}
