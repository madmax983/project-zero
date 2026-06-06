use bevy_ecs::prelude::*;
use crate::layer1::social::SocialInteractionEvent;
use crate::layer1::utility_eval_types::PopEvalData;
use crate::layer1::utility_types::ActionType;

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
                commands.entity(event.target).insert(infection.clone());
            }
        }
    }
}

pub fn evaluate_obsession_utility(
    data: &PopEvalData,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if let Some(_infection) = data.memetic_infection.as_ref() {
        return Some((ActionType::MemeticObsession, 10.0, None));
    }
    None
}

#[cfg(test)]
mod tests {
    use bevy_ecs::system::RunSystemOnce;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::SocialInteractionEvent;
    use crate::layer1::utility_eval_types::PopEvalData;
    use crate::layer1::utility_types::ActionType;
    use crate::layer1::memetics::memetic_plague::{MemeticInfection, ObsessionType, Quarantined, process_memetic_transmission_system, evaluate_obsession_utility};
    use crate::layer1::map::GridPosition;

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

        let healthy_pop = world.spawn((Pop, GridPosition::default())).id();

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

        let infection = world.get::<MemeticInfection>(pop).unwrap();

        let mut data = PopEvalData::test_instance();
        data.memetic_infection = Some(infection.clone());

        let result = evaluate_obsession_utility(&data);

        assert!(result.is_some());
        let (action, utility, _target) = result.unwrap();
        assert_eq!(action, ActionType::MemeticObsession);
        assert!(utility >= 10.0);
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
        )).id();

        let healthy_pop = world.spawn((Pop, Quarantined, GridPosition::default())).id();

        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        world.run_system_once(process_memetic_transmission_system).unwrap();

        assert!(!world.entity(healthy_pop).contains::<MemeticInfection>());
    }
}

#[cfg(test)]
mod additional_coverage {
    use super::*;
    use crate::layer1::utility_eval_types::PopEvalData;

    #[test]
    fn test_obsession_utility_returns_none_when_uninfected() {
        let data = PopEvalData::test_instance();
        let result = evaluate_obsession_utility(&data);
        assert!(result.is_none());
    }
}
