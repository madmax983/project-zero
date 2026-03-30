#![cfg(feature = "nova")]

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::memetics::MemeticInfection;
    use scale::layer1::observatory::Observatory;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::void_signals::{SignalNetwork, SignalReward};

    #[test]
    fn test_decrypt_parasitic_broadcast_infects_colony() {
        // Setup world
        let mut world = World::new();
        world.insert_resource(SignalNetwork::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(scale::shared::log::MessageLog::default());

        // Add ParasiticBroadcast signal
        world.resource_mut::<SignalNetwork>().add_signal(
            "Catchy Tune".into(),
            "A strangely compelling rhythm.".into(),
            SignalReward::ParasiticBroadcast,
        );
        world.resource_mut::<SignalNetwork>().active_signal_id = Some(0);
        world.resource_mut::<SignalNetwork>().signals[0].progress = 99.95;

        // Spawn Observatory & Worker
        let observatory = world.spawn(Observatory { efficiency: 100.0 }).id();
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: observatory,
                    assignment_type: AssignmentType::ObservatoryWorker,
                },
            ))
            .id();

        // Check not infected
        assert!(world.get::<MemeticInfection>(pop).is_none());

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(scale::layer1::void_signals::decrypt_signals_system);
        schedule.run(&mut world);

        // Check signal removed
        let net = world.resource::<SignalNetwork>();
        assert!(net.signals.is_empty());

        // Check pop got infected
        let infection = world.get::<MemeticInfection>(pop).unwrap();
        assert_eq!(*infection, MemeticInfection::ParasiticBroadcast);
    }
}
