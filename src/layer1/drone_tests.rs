#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::drone::{Drone, DroneHub, DroneBattery, evaluate_drone_actions_system, drone_battery_system, process_charge_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::energy::PowerConsumer;

    #[test]
    fn test_drone_component_initialization() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            DroneBattery { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        let battery = world.get::<DroneBattery>(drone).unwrap();
        assert!((battery.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_drone_seeks_charge_when_low() {
        let mut world = World::new();
        // Setup Drone with low battery
        let drone = world.spawn((
            Drone,
            DroneBattery { current: 10.0, max: 100.0 }, // 10%
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        // Setup Hub (Charger)
        let _hub = world.spawn((
            Building { building_type: BuildingType::DroneHub },
            DroneHub,
            GridPosition { x: 5, y: 5 },
            PowerConsumer { demand: 10.0, active: true },
        )).id();

        // Run evaluation
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_drone_actions_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Charge);
    }

    #[test]
    fn test_drone_idle_when_no_task() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            DroneBattery { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_drone_actions_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Idle);
    }

    #[test]
    fn test_drone_charges_at_hub() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            DroneBattery { current: 10.0, max: 100.0 },
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Charge,
                ..Default::default()
            },
        )).id();

        // Setup Hub
        world.spawn((
            Building { building_type: BuildingType::DroneHub },
            DroneHub,
            GridPosition { x: 5, y: 5 },
            PowerConsumer { demand: 10.0, active: true },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_charge_system);
        schedule.run(&mut world);

        let battery = world.get::<DroneBattery>(drone).unwrap();
        assert!(battery.current > 10.0, "Battery should increase");
    }

    #[test]
    fn test_drone_battery_drains() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            DroneBattery { current: 100.0, max: 100.0 },
            PopAction::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(drone_battery_system);
        schedule.run(&mut world);

        let battery = world.get::<DroneBattery>(drone).unwrap();
        assert!(battery.current < 100.0, "Battery should drain");
    }
}
