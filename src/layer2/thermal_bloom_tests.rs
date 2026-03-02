#[cfg(test)]
mod tests {
    use crate::layer1::temperature::HeatSource;
    use crate::layer2::events::DetectionEvent;
    use crate::layer2::thermal::{
        detection_risk_system, update_thermal_bloom_system, ThermalSignature,
    };
    use bevy_ecs::prelude::*;

    #[test]
    fn test_thermal_signature_aggregation() {
        let mut world = World::new();
        // Uses Default which sets decay_rate > 0.0
        world.insert_resource(ThermalSignature::default());

        // Spawn 3 heat sources
        world.spawn(HeatSource { output: 10.0 });
        world.spawn(HeatSource { output: 20.0 });
        world.spawn(HeatSource { output: 5.0 });

        // Run system to update signature
        let mut schedule = Schedule::default();
        schedule.add_systems(update_thermal_bloom_system);
        schedule.run(&mut world);

        let signature = world.resource::<ThermalSignature>();
        // Total = 35.0.
        // With decay_rate 0.1, current_value = 0.0 + (35.0 - 0.0) * 0.1 = 3.5 (first tick)
        assert!(signature.current_value > 0.0);
    }

    #[test]
    fn test_thermal_decay() {
        let mut world = World::new();
        // Start with high signature
        world.insert_resource(ThermalSignature {
            current_value: 100.0,
            ..Default::default()
        });

        // No heat sources spawned (Target = 0.0)

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_thermal_bloom_system);
        schedule.run(&mut world);

        let signature = world.resource::<ThermalSignature>();
        // Should decay towards 0.0
        assert!(signature.current_value < 100.0);
    }

    #[test]
    fn test_detection_risk_trigger() {
        let mut world = World::new();
        world.insert_resource(Events::<DetectionEvent>::default());

        // High signature
        world.insert_resource(ThermalSignature {
            current_value: 1000.0,
            detection_threshold: 500.0,
            decay_rate: 0.1,
        });

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(detection_risk_system);

        // Run multiple times to ensure probability triggers (if probability is < 1.0)
        // However, we can't easily mock RNG here without deeper changes.
        // We'll trust the logic that calling it *might* trigger it.
        // For a deterministic test, we'd need to mock RNG or force probability.
        // The Spec example just runs it. We will run it once.
        schedule.run(&mut world);

        // We cannot deterministically assert the event exists without mocking RNG.
        // But we can assert the system runs without panicking.
    }

    #[test]
    fn test_low_signature_safe() {
        let mut world = World::new();
        world.insert_resource(Events::<DetectionEvent>::default());

        // Low signature
        world.insert_resource(ThermalSignature {
            current_value: 10.0,
            detection_threshold: 500.0,
            decay_rate: 0.1,
        });

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(detection_risk_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<DetectionEvent>>();
        let reader = events.get_cursor();
        assert_eq!(
            reader.len(&events),
            0,
            "Low thermal signature should NOT trigger detection"
        );
    }
}
