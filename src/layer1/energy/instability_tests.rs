#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{
        PowerSource, PowerConsumer, Battery, Conduit,
        power_grid_system
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health; // From Spec 034/071

    // 1. Battery Logic
    #[test]
    fn test_battery_storage() {
        let mut battery = Battery {
            capacity: 100.0,
            charge: 0.0,
            max_throughput: 20.0, // Increased to allow 20.0 discharge
        };

        // Charge
        battery.charge(50.0);
        assert_eq!(battery.charge, 50.0);

        // Overcharge
        battery.charge(100.0);
        assert_eq!(battery.charge, 100.0);

        // Discharge
        let drained = battery.discharge(20.0);
        assert_eq!(drained, 20.0);
        assert_eq!(battery.charge, 80.0);
    }

    // 2. Grid Buffering
    #[test]
    fn test_battery_buffers_shortage() {
        let mut world = World::new();

        // Generator: 10 Output
        world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // Consumer: 15 Demand
        let consumer = world.spawn((
            PowerConsumer { demand: 15.0, active: false },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Battery: 100 Charge
        world.spawn((
            Battery { capacity: 100.0, charge: 100.0, max_throughput: 10.0 },
            GridPosition { x: 0, y: 2 },
            Conduit, // Connects to grid
        ));

        // Run system
        power_grid_system(&mut world);

        // Consumer should be ACTIVE because Battery covered the 5.0 deficit
        let state = world.get::<PowerConsumer>(consumer).unwrap();
        assert!(state.active);

        // Battery should be drained by 5.0
        let battery = world.query::<&Battery>().single(&world);
        assert_eq!(battery.charge, 95.0);
    }

    // 3. Brownout (Partial Activation)
    #[test]
    fn test_brownout_flickering() {
        let mut world = World::new();

        // Generator: 10 Output
        world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // 10 Consumers: 2 Demand each (Total 20)
        // Deficit: 10 Supply vs 20 Demand -> 50% Supply Ratio
        let mut consumers = Vec::new();
        for i in 0..10 {
            consumers.push(world.spawn((
                PowerConsumer { demand: 2.0, active: true }, // Start active
                GridPosition { x: 0, y: i + 1 },
            )).id());
        }

        // Run system
        power_grid_system(&mut world);

        // Check activation count
        // Should be roughly 50% (5 consumers) active
        // Allow variance for RNG, but ensure SOME are off and SOME are on
        let active_count = consumers.iter()
            .filter(|&e| world.get::<PowerConsumer>(*e).unwrap().active)
            .count();

        assert!(active_count < 10, "Not all consumers should be active");
        assert!(active_count > 0, "Some consumers should be active");
    }

    // 4. Overload Damage
    #[test]
    fn test_overload_damage() {
        let mut world = World::new();

        // Generator: 10 Output
        let generator = world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
            Health { current: 100.0, max: 100.0 }, // Has Health
        )).id();

        // Consumer: 30 Demand (300% Load) -> Severe Overload
        world.spawn((
            PowerConsumer { demand: 30.0, active: true },
            GridPosition { x: 0, y: 1 },
        ));

        // Run system multiple times to trigger probability
        for _ in 0..100 {
            power_grid_system(&mut world);
        }

        // Generator should have taken damage
        let health = world.get::<Health>(generator).unwrap();
        assert!(health.current < 100.0, "Generator should take damage from 300% overload");
    }
}
