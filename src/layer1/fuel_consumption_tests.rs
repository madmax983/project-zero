#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::{process_fuel_consumption_system, FuelConsumer, PowerSource};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_power_source_default_active() {
        // PowerSource should default to active
        let source = PowerSource {
            output: 10.0,
            ..Default::default()
        };
        assert!(source.active);
    }

    #[test]
    fn test_fuel_consumption_success() {
        let mut world = World::new();
        // Setup resources with fuel
        world.insert_resource(ColonyResources {
            fuel: 5.0,
            ..ColonyResources::zeroed()
        });

        // Spawn Generator with FuelConsumer
        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                FuelConsumer { amount: 1.0 },
                Building {
                    building_type: BuildingType::Generator,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        world
            .run_system_once(process_fuel_consumption_system)
            .unwrap();

        // Verify fuel consumed
        let res = world.resource::<ColonyResources>();
        // Assert fuel decreased
        assert!(
            (res.fuel - 4.0).abs() < f32::EPSILON,
            "Expected 4.0 fuel, got {}",
            res.fuel
        );

        // Verify generator still active
        let source = world.get::<PowerSource>(generator).unwrap();
        assert!(source.active, "Generator should remain active");
    }

    #[test]
    fn test_fuel_consumption_failure_no_fuel() {
        let mut world = World::new();
        // Setup resources with NO fuel
        world.insert_resource(ColonyResources {
            fuel: 0.0,
            ..ColonyResources::zeroed()
        });

        // Spawn Generator
        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                FuelConsumer { amount: 1.0 },
                Building {
                    building_type: BuildingType::Generator,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        world
            .run_system_once(process_fuel_consumption_system)
            .unwrap();

        // Verify fuel unchanged (0)
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.fuel, 0.0);

        // Verify generator INACTIVE
        let source = world.get::<PowerSource>(generator).unwrap();
        assert!(
            !source.active,
            "Generator should be inactive due to lack of fuel"
        );
    }

    #[test]
    fn test_fuel_consumption_reactivation() {
        let mut world = World::new();
        // Start with no fuel
        world.insert_resource(ColonyResources {
            fuel: 0.0,
            ..ColonyResources::zeroed()
        });

        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: false,
                }, // Previously disabled
                FuelConsumer { amount: 1.0 },
                Building {
                    building_type: BuildingType::Generator,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Add fuel
        world.resource_mut::<ColonyResources>().fuel = 10.0;

        // Run system
        world
            .run_system_once(process_fuel_consumption_system)
            .unwrap();

        // Verify active
        let source = world.get::<PowerSource>(generator).unwrap();
        assert!(
            source.active,
            "Generator should reactivate when fuel is available"
        );
    }

    #[test]
    fn test_fuel_consumption_inactive_remains_inactive_no_fuel() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            fuel: 0.0,
            ..ColonyResources::zeroed()
        });

        let generator = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: false,
                },
                FuelConsumer { amount: 1.0 },
                Building {
                    building_type: BuildingType::Generator,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        world
            .run_system_once(process_fuel_consumption_system)
            .unwrap();

        let source = world.get::<PowerSource>(generator).unwrap();
        assert!(
            !source.active,
            "Generator should remain inactive when no fuel available"
        );

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.fuel, 0.0);
    }
}
