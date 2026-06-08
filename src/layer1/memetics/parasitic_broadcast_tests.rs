#[cfg(test)]
mod tests {
    use crate::layer1::execution::general_work::calculate_work_amount;
    use crate::layer1::memetics::memetic_hazards::{
        process_memetic_transmission_system, ConversationEvent,
    };
    use crate::layer1::memetics::parasitic_broadcast::{ObsessionType, Quarantined};
    use crate::layer1::memetics::{
        parasitic_broadcast_risk_system, process_parasitic_work_reduction, MemeticInfection,
    };
    use crate::layer1::morale::{MoodModifier, Morale};
    use crate::layer1::skills::Skills;
    use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
    use crate::layer1::utility_types::ActionType;
    use crate::layer1::DesignationType;
    use crate::layer3::silence::DetectionRisk;
    use crate::prelude::Pop;
    use bevy::prelude::App;
    use bevy::prelude::Update;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_parasitic_broadcast_increases_morale_and_reduces_work() {
        let mut world = World::new();
        // Setup a pop with the broadcast infection
        let pop = world
            .spawn((
                Morale {
                    value: 50.0,
                    ..Default::default()
                },
                MemeticInfection::default(), // Humming the catchy tune
                Skills::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_parasitic_work_reduction);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();

        // Massive morale boost
        assert!(morale.value > 50.0, "The song is extremely catchy");
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Entertained (Parasitic Broadcast)"));

        let infected_work_amount =
            calculate_work_amount(&world, pop, DesignationType::Mine, None, 50.0, 1.0, 1.0);

        world.entity_mut(pop).remove::<MemeticInfection>();

        let uninfected_work_amount =
            calculate_work_amount(&world, pop, DesignationType::Mine, None, 50.0, 1.0, 1.0);

        // Due to random variance, checking bounds instead of strict equality
        assert!(
            infected_work_amount < uninfected_work_amount * 0.7,
            "Work output should be roughly halved"
        );
    }

    #[test]
    fn test_infected_pops_increase_detection_risk() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk {
            current_risk: 0.0,
            threshold: 100.0,
        });

        // Spawn 10 infected pops
        for _ in 0..10 {
            world.spawn(MemeticInfection::default());
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(parasitic_broadcast_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        assert!(
            risk.current_risk > 0.0,
            "Pops are rewiring machines to broadcast into space, increasing detection risk"
        );
    }

    #[test]
    fn test_parasitic_broadcast_process_no_infection() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Morale {
                    value: 50.0,
                    ..Default::default()
                },
                Skills::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_parasitic_work_reduction);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();

        assert_eq!(morale.value, 50.0);
        assert!(morale.modifiers.is_empty());
    }

    #[test]
    fn test_parasitic_broadcast_risk_empty() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk {
            current_risk: 0.0,
            threshold: 100.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(parasitic_broadcast_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        assert_eq!(risk.current_risk, 0.0);
    }

    #[test]
    fn test_parasitic_broadcast_process_already_has_modifier() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Morale {
                    value: 50.0,
                    modifiers: vec![MoodModifier {
                        label: "Entertained (Parasitic Broadcast)".to_string(),
                        value: 10.0,
                        duration: 1,
                    }],
                },
                MemeticInfection::default(),
                Skills::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_parasitic_work_reduction);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();

        assert_eq!(morale.value, 60.0);
        assert_eq!(morale.modifiers.len(), 1);
    }

    #[test]
    fn test_memetic_infection_spreads_via_conversation() {
        let mut app = App::new();
        app.add_systems(Update, process_memetic_transmission_system);
        app.add_event::<ConversationEvent>();

        // Arrange
        let carrier = app
            .world_mut()
            .spawn((
                Pop,
                MemeticInfection {
                    obsession_type: ObsessionType::DigHoles,
                    intensity: 1.0,
                },
            ))
            .id();
        let target = app.world_mut().spawn((Pop,)).id();

        app.world_mut().send_event(ConversationEvent {
            initiator: carrier,
            receiver: target,
        });

        // Act
        app.update();

        // Assert
        assert!(
            app.world().get::<MemeticInfection>(target).is_some(),
            "Target pop should contract the virus after conversation with a carrier"
        );
        assert_eq!(
            app.world()
                .get::<MemeticInfection>(target)
                .unwrap()
                .obsession_type,
            ObsessionType::DigHoles
        );
    }

    #[test]
    fn test_memetic_infection_spreads_via_conversation_receiver_to_initiator() {
        let mut app = App::new();
        app.add_systems(Update, process_memetic_transmission_system);
        app.add_event::<ConversationEvent>();

        // Arrange
        let target = app.world_mut().spawn((Pop,)).id();
        let carrier = app
            .world_mut()
            .spawn((
                Pop,
                MemeticInfection {
                    obsession_type: ObsessionType::StackChairs,
                    intensity: 1.0,
                },
            ))
            .id();

        app.world_mut().send_event(ConversationEvent {
            initiator: target,
            receiver: carrier,
        });

        // Act
        app.update();

        // Assert
        assert!(
            app.world().get::<MemeticInfection>(target).is_some(),
            "Initiator pop should contract the virus after conversation with a carrier receiver"
        );
        assert_eq!(
            app.world()
                .get::<MemeticInfection>(target)
                .unwrap()
                .obsession_type,
            ObsessionType::StackChairs
        );
    }

    #[test]
    fn test_quarantine_prevents_transmission() {
        let mut app = App::new();
        app.add_systems(Update, process_memetic_transmission_system);
        app.add_event::<ConversationEvent>();

        let infected_pop = app
            .world_mut()
            .spawn((
                Pop,
                MemeticInfection {
                    obsession_type: ObsessionType::DigHoles,
                    intensity: 1.0,
                },
                Quarantined, // Marker preventing social interaction
            ))
            .id();

        let healthy_pop = app.world_mut().spawn((Pop,)).id();

        app.world_mut().send_event(ConversationEvent {
            initiator: infected_pop,
            receiver: healthy_pop,
        });

        app.update();

        // The healthy pop should remain uninfected due to quarantine
        assert!(!app
            .world()
            .entity(healthy_pop)
            .contains::<MemeticInfection>());
    }

    #[test]
    fn test_infected_pop_prioritizes_obsession_task() {
        let mut data = PopEvalData::test_instance();
        data.memetic_infection = Some(MemeticInfection {
            obsession_type: ObsessionType::DigHoles,
            intensity: 1.0,
        });

        let buffer = UtilityAIBuffer::default();

        let result = crate::layer1::memetics::parasitic_broadcast::evaluate_memetic_obsession(
            &data, &buffer,
        );
        assert!(result.is_some());

        let (action, utility, _target) = result.unwrap();
        assert_eq!(action, ActionType::MemeticObsession);
        assert_eq!(utility, 10.0);
    }
    #[test]
    fn test_evaluate_memetic_obsession_no_infection() {
        let mut data = PopEvalData::test_instance();
        data.memetic_infection = None;
        let buffer = UtilityAIBuffer::default();
        let result = crate::layer1::memetics::parasitic_broadcast::evaluate_memetic_obsession(
            &data, &buffer,
        );
        assert!(result.is_none());
    }
}
