use super::*;
use crate::layer1::pop::Pop;
use crate::layer1::social::SocialInteractionEvent;
use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use bevy_ecs::system::RunSystemOnce;

fn setup_world() -> World {
    let mut world = World::new();
    world.init_resource::<Events<SocialInteractionEvent>>();
    world
}

#[test]
fn test_memetic_infection_spread_via_social_interaction() {
    let mut world = setup_world();

    let infected_pop = world
        .spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::StackChairs,
                intensity: 1.0,
            },
        ))
        .id();

    let healthy_pop = world.spawn((Pop,)).id();

    // Simulate a social interaction
    world.send_event(SocialInteractionEvent {
        initiator: infected_pop,
        target: healthy_pop,
    });

    // Run transmission system
    world
        .run_system_once(process_memetic_transmission_system)
        .unwrap();

    // Assert the healthy pop is now infected
    assert!(world.entity(healthy_pop).contains::<MemeticInfection>());
}

#[test]
fn test_infected_pop_prioritizes_obsession_task() {
    let mut pop_data = PopEvalData::test_instance();
    pop_data.memetic_infection = Some(MemeticInfection {
        obsession_type: ObsessionType::DigHoles,
        intensity: 1.0,
    });

    let buffer = UtilityAIBuffer::default();

    // Run AI evaluation
    let result = evaluate_memetic_obsession(&pop_data, &buffer);

    // The obsession task should have high utility (e.g. 10.0)
    assert_eq!(result, Some((ActionType::MemeticObsession, 10.0, None)));
}

#[test]
fn test_quarantine_prevents_transmission() {
    let mut world = setup_world();

    let infected_pop = world
        .spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },
            Quarantined, // Marker preventing social interaction
        ))
        .id();

    let healthy_pop = world.spawn((Pop,)).id();

    world.send_event(SocialInteractionEvent {
        initiator: infected_pop,
        target: healthy_pop,
    });

    world
        .run_system_once(process_memetic_transmission_system)
        .unwrap();

    // The healthy pop should remain uninfected due to quarantine
    assert!(!world.entity(healthy_pop).contains::<MemeticInfection>());
}
