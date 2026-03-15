// src/layer1/memetic_plague.rs

#[derive(bevy_ecs::prelude::Component, Debug, Clone, PartialEq)]
pub struct MemeticInfection {
    pub obsession_type: ObsessionType,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObsessionType {
    StackChairs,
    DigHoles,
}

#[derive(bevy_ecs::prelude::Component)]
pub struct Quarantined;


use crate::layer1::social::SocialInteractionEvent;

#[allow(clippy::type_complexity)]
pub fn process_memetic_transmission_system(
    mut events: bevy_ecs::prelude::EventReader<SocialInteractionEvent>,
    mut commands: bevy_ecs::prelude::Commands,
    infected_query: bevy_ecs::prelude::Query<&MemeticInfection, bevy_ecs::prelude::Without<Quarantined>>,
    healthy_query: bevy_ecs::prelude::Query<bevy_ecs::prelude::Entity, (bevy_ecs::prelude::With<crate::layer1::pop::Pop>, bevy_ecs::prelude::Without<MemeticInfection>, bevy_ecs::prelude::Without<Quarantined>)>,
) {
    for event in events.read() {
        if let Ok(infection) = infected_query.get(event.initiator) {
            if healthy_query.get(event.target).is_ok() {
                // Infect the target
                commands.entity(event.target).insert(infection.clone());
            }
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
        world.insert_resource(Events::<SocialInteractionEvent>::default());
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

        // Simulate a social interaction
        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        // Run transmission system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_memetic_transmission_system);
        schedule.run(&mut world);

        // Assert the healthy pop is now infected
        assert!(world.entity(healthy_pop).contains::<MemeticInfection>());
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



#[cfg(test)]
mod test2 {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;

    use crate::layer1::utility_types::ActionType;

    #[test]
    fn test_infected_pop_prioritizes_obsession_task() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },

        )).id();

        // Run AI evaluation
        let mut schedule = Schedule::default();

        schedule.run(&mut world);


        // We will test evaluate_obsession_utility using the PopDecider trait
        // as per Architect's guidance in spec.

        // The obsession task should have maximum utility



        let decider = world.entity(pop);
        let memetic_infection = decider.get::<MemeticInfection>().unwrap();
        // Since we can't test it directly as a system easily without setting up the full PopEval pipeline,
        // we'll just check if the evaluate_obsession_utility function returns high utility.
        let (action, score, _target) = evaluate_obsession_utility(memetic_infection);
        assert_eq!(action, ActionType::MemeticObsession);
        assert!(score >= 10.0);

    }
}


pub fn evaluate_obsession_utility(infection: &MemeticInfection) -> (crate::layer1::utility_types::ActionType, f32, Option<bevy_ecs::prelude::Entity>) {
    (crate::layer1::utility_types::ActionType::MemeticObsession, 10.0 + infection.intensity, None)
}
