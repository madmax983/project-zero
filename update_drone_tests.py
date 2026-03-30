import re

with open("src/layer1/drone_tests.rs", "r") as f:
    content = f.read()

content = content.replace(
    "use crate::layer1::drone::{\n        drone_battery_system, evaluate_drone_actions_system, process_charge_system, Drone,\n        DroneBattery, DroneHub,\n    };",
    "use crate::layer1::drone::{\n        drone_battery_system, evaluate_drone_actions_system, process_charge_system, drone_power_monitor_system, Drone,\n        DroneBattery, DroneHub, ParentHub,\n    };"
)

content = content.replace(
    "        assert!(battery.current < 100.0, \"Battery should drain\");\n    }\n}",
    """        assert!(battery.current < 100.0, "Battery should drain");
    }

    #[test]
    fn test_drones_deactivate_without_power_or_bandwidth() {
        let mut world = World::new();

        let hub = world
            .spawn((
                Building {
                    building_type: BuildingType::DroneHub,
                },
                DroneHub,
                GridPosition { x: 5, y: 5 },
                PowerConsumer {
                    demand: 10.0,
                    active: false, // Power cut!
                },
            ))
            .id();

        let drone = world
            .spawn((
                Drone,
                ParentHub(hub),
                DroneBattery {
                    current: 50.0,
                    max: 100.0,
                },
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(drone_power_monitor_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Idle, "Drone should be inactive (Idle) when parent hub loses power.");
    }
}"""
)

with open("src/layer1/drone_tests.rs", "w") as f:
    f.write(content)
