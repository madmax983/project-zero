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
                Drone { state: DroneState::Idle },
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
                Drone { state: DroneState::Idle },
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
                Drone { state: DroneState::Idle },
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
                Drone { state: DroneState::Idle },
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
                Drone { state: DroneState::Idle },
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
        use crate::layer1::drone::{DroneDisconnectedEvent, check_drone_connection, process_feral_drones};
        use bevy_app::Update;
        let mut app = bevy_app::App::new();
        app.add_event::<DroneDisconnectedEvent>();
        app.add_systems(Update, (check_drone_connection, process_feral_drones));
        app
    }

    #[test]
    fn test_drone_becomes_feral_on_disconnect() {
        use crate::layer1::drone::{ConnectedTo, FeralDrone, DroneState};
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::energy::PowerConsumer;
        use crate::layer1::drone::Drone;

        let mut app = setup_app();
        let command_center = app.world_mut().spawn((
            Building { building_type: BuildingType::CommandCenter },
            PowerConsumer { demand: 10.0, active: true },
        )).id();

        let drone = app.world_mut().spawn((
            Drone { state: DroneState::Hauling },
            ConnectedTo(command_center),
        )).id();

        // Act - Simulate power loss leading to disconnect
        app.world_mut().entity_mut(command_center).remove::<PowerConsumer>();
        app.update(); // check_drone_connection runs

        // Assert
        assert!(app.world().entity(drone).contains::<FeralDrone>());
        assert_eq!(app.world().get::<Drone>(drone).unwrap().state, DroneState::Feral);
    }

    #[test]
    fn test_feral_drone_hoards_resources() {
        use crate::layer1::drone::{FeralDrone, DroneState, Drone};
        use crate::layer1::resources::{ResourceItem, ResourceType};
        use crate::layer1::map::GridPosition;

        // Arrange
        let mut app = setup_app();
        let drone = app.world_mut().spawn((
            Drone { state: DroneState::Feral },
            FeralDrone { hoard: vec![] },
            GridPosition { x: 0, y: 0 },
        )).id();

        let resource = app.world_mut().spawn((
            ResourceItem { resource_type: ResourceType::Metal, amount: 10.0 },
            GridPosition { x: 1, y: 0 },
        )).id();

        // Act
        app.update(); // process_feral_drones runs

        // Assert - The drone picked up the nearby resource
        let feral_drone = app.world().get::<FeralDrone>(drone).unwrap();
        assert!(!feral_drone.hoard.is_empty());
        assert!(app.world().get_entity(resource).is_err()); // Resource removed from ground
    }

    #[test]
    fn test_feral_drone_attacks_nearby_pops() {
        use crate::layer1::drone::{FeralDrone, DroneState, Drone};
        use crate::layer1::pop::Pop;
        use crate::layer1::health::Health;
        use crate::layer1::map::GridPosition;

        // Arrange
        let mut app = setup_app();
        let _drone = app.world_mut().spawn((
            Drone { state: DroneState::Feral },
            FeralDrone::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 1, y: 0 },
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(pop).unwrap();
        assert!(health.current < health.max, "Pop should have taken damage from Feral Drone");
    }
}
