use bevy_ecs::prelude::*;
use crate::layer1::social::SocialInteractionEvent;
use crate::layer1::utility_types::ActionType;
use crate::layer1::pop::Pop;

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
    healthy_query: Query<Entity, (With<Pop>, Without<MemeticInfection>, Without<Quarantined>)>,
) {
    for event in events.read() {
        if let Ok(infection) = infected_query.get(event.initiator) {
            if healthy_query.get(event.target).is_ok() {
                commands.entity(event.target).insert(infection.clone());
            }
        }
    }
}

pub fn evaluate_obsession_utility(
    _infection: &MemeticInfection,
) -> Option<(ActionType, f32, Option<Entity>)> {
    // Utility score 10.0 overrides basic needs.
    Some((ActionType::MemeticObsession, 10.0, None))
}

pub fn evaluate_obsession_utility_system(
    query: Query<&MemeticInfection>,
) {
    // Kept to make tests pass initially based on spec layout, but we will wire the real AI via PopDecider.
    for infection in query.iter() {
        if let Some((_action_type, _utility, _target)) = evaluate_obsession_utility(infection) {
            // we do nothing with the buffer here, we're returning it via PopDecider
        }
    }
}

pub fn memetic_obsession_execution_system(
    mut query: Query<(&mut crate::layer1::utility_types::PopAction, &MemeticInfection)>,
) {
    for (mut action, _infection) in query.iter_mut() {
        if action.current == ActionType::MemeticObsession {
            // Just tick execution, the Pop might wander or stay put.
            action.ticks_committed += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
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

        let healthy_pop = world.spawn((Pop,)).id();

        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_memetic_transmission_system);
        schedule.run(&mut world);

        assert!(world.entity(healthy_pop).contains::<MemeticInfection>());
    }

    #[test]
    fn test_infected_pop_prioritizes_obsession_task() {
        let infection = MemeticInfection {
            obsession_type: ObsessionType::DigHoles,
            intensity: 1.0,
        };

        let result = evaluate_obsession_utility(&infection);
        assert!(result.is_some());
        let (action_type, utility, _target) = result.unwrap();
        assert_eq!(action_type, ActionType::MemeticObsession);
        assert_eq!(utility, 10.0);
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
            Quarantined, // Marker preventing social interaction
        )).id();

        let healthy_pop = world.spawn((Pop,)).id();

        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_memetic_transmission_system);
        schedule.run(&mut world);

        // The healthy pop should remain uninfected due to quarantine
        assert!(!world.entity(healthy_pop).contains::<MemeticInfection>());
    }
}