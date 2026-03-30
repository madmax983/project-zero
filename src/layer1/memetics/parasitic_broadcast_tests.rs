#[cfg(test)]
mod tests {
    use crate::layer1::execution::general_work::calculate_work_amount;
    use crate::layer1::memetics::{
        parasitic_broadcast_risk_system, process_parasitic_work_reduction,
        ParasiticBroadcastInfection,
    };
    use crate::layer1::morale::Morale;
    use crate::layer1::skills::Skills;
    use crate::layer1::DesignationType;
    use crate::layer3::silence::DetectionRisk;
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
                ParasiticBroadcastInfection, // Humming the catchy tune
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

        world
            .entity_mut(pop)
            .remove::<ParasiticBroadcastInfection>();

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
            world.spawn(ParasiticBroadcastInfection);
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
}
