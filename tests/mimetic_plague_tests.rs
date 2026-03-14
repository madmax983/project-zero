#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::Pop;
    use scale::layer1::utility_types::{ActionType, UtilityAIBuffer};
    use scale::layer1::memetic_plague::{
        MemeticInfection, ObsessionType, Quarantined, process_memetic_transmission_system, evaluate_obsession_utility_system
    };
    use scale::layer1::social::SocialInteractionEvent;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems and resources
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
    fn test_infected_pop_prioritizes_obsession_task() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },
            UtilityAIBuffer::default(),
        )).id();

        // Run AI evaluation
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_obsession_utility_system);
        schedule.run(&mut world);

        let buffer = world.get::<UtilityAIBuffer>(pop).unwrap();
        // The obsession task should have maximum utility
        // Assuming we evaluate by scanning the buffer array:
        let top_action_opt = buffer.get_top_action();
        assert!(top_action_opt.is_some());
        let top_action = top_action_opt.unwrap();
        assert_eq!(top_action.action_type, ActionType::MemeticObsession);
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

        let healthy_pop = world.spawn((
            Pop,
            Quarantined, // Marker preventing social interaction
        )).id();

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
