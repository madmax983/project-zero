use bevy_ecs::prelude::*;
use crate::layer1::memetics::MemeticInfection;
use crate::layer3::silence::DetectionRisk;

/// Bridges Layer 1 `MemeticInfection::ParasiticBroadcast` to Layer 3 `DetectionRisk`.
/// Every infected pop acts as a tiny antenna, accumulating detection risk.
pub fn parasitic_broadcast_risk_system(
    mut risk: ResMut<DetectionRisk>,
    query: Query<(), With<MemeticInfection>>,
) {
    let infected_count = query.iter().count() as f32;
    risk.current_risk += infected_count * 0.1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::memetics::MemeticInfection;
    use crate::layer3::silence::DetectionRisk;

    #[test]
    fn test_infected_pops_increase_detection_risk() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk {
            current_risk: 0.0,
            threshold: 100.0,
        });

        // Spawn 10 infected pops
        for _ in 0..10 {
            world.spawn(MemeticInfection::ParasiticBroadcast);
        }

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(parasitic_broadcast_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        assert!(
            risk.current_risk > 0.0,
            "Pops are rewiring machines to broadcast into space, increasing detection risk"
        );
    }
}
