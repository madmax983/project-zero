#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::memetics::parasitic_broadcast::{MemeticInfection, process_parasitic_work_reduction, parasitic_broadcast_risk_system};
    use crate::layer1::morale::Morale;
    use crate::layer1::jobs::WorkEfficiency;
    use crate::layer3::silence::DetectionRisk;

    #[test]
    fn test_parasitic_broadcast_increases_morale_and_reduces_work() {
        let mut world = World::new();
        // Setup a pop with the broadcast infection
        let pop = world.spawn((
            Morale::default(),
            WorkEfficiency { multiplier: 1.0 },
            MemeticInfection::ParasiticBroadcast, // Humming the catchy tune
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_parasitic_work_reduction);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        let eff = world.get::<WorkEfficiency>(pop).unwrap();

        // Massive morale boost via modifier
        let has_boost = morale.modifiers.iter().any(|m| m.label == "Catchy Tune" && m.value > 0.0);
        assert!(has_boost, "The song is extremely catchy");

        // Massive productivity loss
        assert!(eff.multiplier < 1.0, "They are too busy humming to work");
    }

    #[test]
    fn test_infected_pops_increase_detection_risk() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk { current_risk: 0.0, threshold: 100.0 });

        // Spawn 10 infected pops
        for _ in 0..10 {
            world.spawn(MemeticInfection::ParasiticBroadcast);
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(parasitic_broadcast_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        assert!(risk.current_risk > 0.0, "Pops are rewiring machines to broadcast into space, increasing detection risk");
    }
}
