#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::SystemBody;
    use crate::layer2::debris::OrbitalDebris;
    use crate::layer2::events::{LaunchEvent, ShipDestroyedEvent};
    use crate::layer2::fleet::{Fleet, InOrbit, FleetHealth};
    use crate::layer2::debris::{debris_accumulation_system, debris_attrition_system, debris_decay_system};

    #[test]
    fn test_debris_decay() {
        let mut world = World::new();
        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 1.0 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(debris_decay_system);
        schedule.run(&mut world);

        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        assert!(debris.amount < 1.0, "Debris should decay over time");
        assert!(debris.amount > 0.9, "Decay should be slow");
    }

    #[test]
    fn test_debris_accumulation_on_launch() {
        let mut world = World::new();
        // Register events
        world.init_resource::<Events<LaunchEvent>>();
        world.init_resource::<Events<ShipDestroyedEvent>>();

        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.0 })).id();

        // Trigger Launch Event
        world.send_event(LaunchEvent { planet, success: true });

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(debris_accumulation_system);
        schedule.run(&mut world);

        // Verify Debris Increase
        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        assert!(debris.amount > 0.0, "Debris should increase after launch");
    }

    #[test]
    fn test_debris_accumulation_on_ship_destruction() {
        let mut world = World::new();
        world.init_resource::<Events<LaunchEvent>>();
        world.init_resource::<Events<ShipDestroyedEvent>>();

        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.0 })).id();

        // Trigger Destruction Event
        world.send_event(ShipDestroyedEvent { planet, ship_class: "Frigate".to_string() });

        let mut schedule = Schedule::default();
        schedule.add_systems(debris_accumulation_system);
        schedule.run(&mut world);

        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        // Should be higher than launch debris (0.05 vs 0.20 roughly)
        assert!(debris.amount >= 0.1, "Debris should increase significantly after ship destruction");
    }

    #[test]
    fn test_orbit_attrition_damage() {
        let mut world = World::new();
        // Register events required by system
        world.init_resource::<Events<ShipDestroyedEvent>>();

        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.5 })).id(); // High debris

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetHealth { current: 100.0, max: 100.0 }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(debris_attrition_system);
        schedule.run(&mut world);

        let health = world.get::<FleetHealth>(fleet).unwrap();
        assert!(health.current < 100.0, "Fleet should take damage from debris");
    }

    #[test]
    fn test_fleet_destruction_from_debris() {
        let mut world = World::new();
        world.init_resource::<Events<ShipDestroyedEvent>>();

        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 10.0 })).id(); // Extreme debris

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetHealth { current: 1.0, max: 100.0 }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems((debris_attrition_system, apply_deferred));
        schedule.run(&mut world);

        assert!(world.get::<Fleet>(fleet).is_none(), "Fleet should be destroyed by debris");

        // Verify event emitted
        let events = world.resource::<Events<ShipDestroyedEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).count() > 0, "Should emit destruction event");
    }

    #[test]
    fn test_launch_failure_risk_from_debris() {
        // This requires mocking RNG or checking probability calculation function directly
        let risk = crate::layer2::debris::calculate_launch_risk(0.8); // 80% debris
        assert!(risk > 0.5, "Risk should be high for high debris"); // High risk

        let risk_low = crate::layer2::debris::calculate_launch_risk(0.0);
        assert_eq!(risk_low, 0.0, "Risk should be zero for no debris");
    }

    #[test]
    fn test_debris_cleanup_action() {
        let mut world = World::new();
        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.5 })).id();

        // Assume a Cleanup Action system exists or simulate it
        crate::layer2::debris::perform_cleanup(&mut world, planet, 0.2);

        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        assert!((debris.amount - 0.3).abs() < 0.001, "Cleanup should reduce debris");
    }
}
