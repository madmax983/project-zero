#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::drone::{
        drone_battery_system, evaluate_drone_actions_system, process_charge_system, Drone,
        DroneBattery, DroneHub,
    };
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_drone_component_initialization() {
        use crate::layer1::drone::DroneState;
        let mut world = World::new();
        let drone = world
            .spawn((
                Drone {
                    state: DroneState::Idle,
                },
                DroneBattery {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 0, y: 0 },
                PopAction::default(),
            ))
            .id();

        let battery = world.get::<DroneBattery>(drone).unwrap();
        assert!((battery.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_drone_seeks_charge_when_low() {
        use crate::layer1::drone::DroneState;
        let mut world = World::new();
        // Setup Drone with low battery
        let drone = world
            .spawn((
                Drone {
                    state: DroneState::Idle,
                },
                DroneBattery {
                    current: 10.0,
                    max: 100.0,
                }, // 10%
                GridPosition { x: 0, y: 0 },
                PopAction::default(),
            ))
            .id();

        // Setup Hub (Charger)
        let _hub = world
            .spawn((
                Building {
                    building_type: BuildingType::DroneHub,
                },
                DroneHub,
                GridPosition { x: 5, y: 5 },
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
            ))
            .id();

        // Run evaluation
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_drone_actions_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Charge);
    }

    #[test]
    fn test_drone_idle_when_no_task() {
        use crate::layer1::drone::DroneState;
        let mut world = World::new();
        let drone = world
            .spawn((
                Drone {
                    state: DroneState::Idle,
                },
                DroneBattery {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 0, y: 0 },
                PopAction::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_drone_actions_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Idle);
    }

    #[test]
    fn test_drone_charges_at_hub() {
        use crate::layer1::drone::DroneState;
        let mut world = World::new();
        let drone = world
            .spawn((
                Drone {
                    state: DroneState::Idle,
                },
                DroneBattery {
                    current: 10.0,
                    max: 100.0,
                },
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Charge,
                    ..Default::default()
                },
            ))
            .id();

        // Setup Hub
        world.spawn((
            Building {
                building_type: BuildingType::DroneHub,
            },
            DroneHub,
            GridPosition { x: 5, y: 5 },
            PowerConsumer {
                demand: 10.0,
                active: true,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_charge_system);
        schedule.run(&mut world);

        let battery = world.get::<DroneBattery>(drone).unwrap();
        assert!(battery.current > 10.0, "Battery should increase");
    }

    #[test]
    fn test_drone_battery_drains() {
        use crate::layer1::drone::DroneState;
        let mut world = World::new();
        let drone = world
            .spawn((
                Drone {
                    state: DroneState::Idle,
                },
                DroneBattery {
                    current: 100.0,
                    max: 100.0,
                },
                PopAction::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(drone_battery_system);
        schedule.run(&mut world);

        let battery = world.get::<DroneBattery>(drone).unwrap();
        assert!(battery.current < 100.0, "Battery should drain");
    }

    fn setup_app() -> bevy_app::App {
        use crate::layer1::drone::{
            check_drone_connection, process_feral_drones, DroneDisconnectedEvent,
        };
        use bevy_app::Update;
        let mut app = bevy_app::App::new();
        app.add_event::<DroneDisconnectedEvent>();
        app.add_systems(Update, (check_drone_connection, process_feral_drones));
        app
    }

    #[test]
    fn test_drone_goes_feral_when_disconnected_from_grid() {
        use crate::layer1::drone::{check_feral_state_system, Drone, FeralDrone, GridConnection, PlayerOwned};
        use crate::shared::time::SimulationTime;

        // Arrange
        let mut app = bevy_app::App::new();
        app.add_event::<crate::layer1::drone::DroneDisconnectedEvent>();
        app.init_resource::<SimulationTime>();

        let drone_entity = app.world_mut().spawn((
            Drone::default(),
            PlayerOwned,
            GridConnection { is_connected: false, time_disconnected: 0 },
        )).id();

        app.add_systems(bevy_app::Update, check_feral_state_system);

        // Act: Advance time beyond the feral threshold (e.g., 5000 ticks)
        let mut time = app.world_mut().resource_mut::<SimulationTime>();
        time.tick = 5001;

        let mut connection = app.world_mut().get_mut::<GridConnection>(drone_entity).unwrap();
        connection.time_disconnected = 5001;

        app.update();

        // Assert
        let drone = app.world().entity(drone_entity);
        assert!(drone.contains::<FeralDrone>(), "Drone should become feral after prolonged disconnection");
        assert!(!drone.contains::<PlayerOwned>(), "Feral drone should no longer be player-owned");
    }

    #[test]
    fn test_feral_drone_targets_resources() {
        use crate::layer1::drone::{Drone, FeralDrone, evaluate_feral_actions_system};
        use crate::layer1::utility_ai::{ActionType, PopAction};

        // Arrange
        let mut app = bevy_app::App::new();
        let feral_drone_entity = app.world_mut().spawn((
            Drone::default(),
            FeralDrone::default(),
            PopAction::default(),
            crate::layer1::map::GridPosition { x: 0, y: 0 },
        )).id();

        // Add dummy resource to target
        app.world_mut().spawn((
            crate::layer1::resources::ResourceItem {
                resource_type: crate::layer1::resources::ResourceType::Metal,
                amount: 10.0,
            },
            crate::layer1::map::GridPosition { x: 1, y: 1 },
        ));

        app.add_systems(bevy_app::Update, evaluate_feral_actions_system);

        // Act
        app.update();

        // Assert
        let action = app.world().get::<PopAction>(feral_drone_entity).unwrap();
        assert_eq!(action.current, ActionType::Harvest, "Feral drone should heavily prioritize harvesting for survival");
    }

    #[test]
    fn test_drone_becomes_feral_on_disconnect() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::drone::Drone;
        use crate::layer1::drone::{ConnectedTo, DroneState, FeralDrone};
        use crate::layer1::energy::PowerConsumer;

        let mut app = setup_app();
        let command_center = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::CommandCenter,
                },
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
            ))
            .id();

        let drone = app
            .world_mut()
            .spawn((
                Drone {
                    state: DroneState::Hauling,
                },
                ConnectedTo(command_center),
            ))
            .id();

        // Act - Simulate power loss leading to disconnect
        app.world_mut()
            .entity_mut(command_center)
            .remove::<PowerConsumer>();
        app.update(); // check_drone_connection runs

        // Assert
        assert!(app.world().entity(drone).contains::<FeralDrone>());
        assert_eq!(
            app.world().get::<Drone>(drone).unwrap().state,
            DroneState::Feral
        );
    }

    #[test]
    fn test_feral_drone_hoards_resources() {
        use crate::layer1::drone::{Drone, DroneState, FeralDrone};
        use crate::layer1::map::GridPosition;
        use crate::layer1::resources::{ResourceItem, ResourceType};

        // Arrange
        let mut app = setup_app();
        let drone = app
            .world_mut()
            .spawn((
                Drone {
                    state: DroneState::Feral,
                },
                FeralDrone { hoard: vec![] },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let resource = app
            .world_mut()
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Metal,
                    amount: 10.0,
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Act
        app.update(); // process_feral_drones runs

        // Assert - The drone picked up the nearby resource
        let feral_drone = app.world().get::<FeralDrone>(drone).unwrap();
        assert!(!feral_drone.hoard.is_empty());
        assert!(app.world().get_entity(resource).is_err()); // Resource removed from ground
    }

    #[test]
    fn test_feral_drone_attacks_nearby_pops() {
        use crate::layer1::drone::{Drone, DroneState, FeralDrone};
        use crate::layer1::health::Health;
        use crate::layer1::map::GridPosition;
        use crate::layer1::pop::Pop;

        // Arrange
        let mut app = setup_app();
        let _drone = app
            .world_mut()
            .spawn((
                Drone {
                    state: DroneState::Feral,
                },
                FeralDrone::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(pop).unwrap();
        assert!(
            health.current < health.max,
            "Pop should have taken damage from Feral Drone"
        );
    }
}
