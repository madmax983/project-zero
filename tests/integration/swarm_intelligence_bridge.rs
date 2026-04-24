use bevy::prelude::IntoSystemConfigs;
use scale::layer1::entities::drone::Drone;
use scale::layer1::entities::swarm_intelligence::{
    update_drone_behavior, update_drone_clusters, DroneBehavior, IntelligenceLevel,
};
use scale::layer1::map::GridPosition;
use scale::layer1::utility_ai::{ActionType, PopAction};

#[test]
fn test_swarm_intelligence_dumb_drone() {
    let mut app = bevy_app::App::new();
    app.add_systems(
        bevy_app::Update,
        (update_drone_clusters, update_drone_behavior).chain(),
    );

    let drone = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 100, y: 100 },
            DroneBehavior::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 0,
            },
        ))
        .id();

    app.update();

    let behavior = app.world().get::<DroneBehavior>(drone).unwrap();
    assert_eq!(behavior.intelligence_level, IntelligenceLevel::Low);

    let action = app.world().get::<PopAction>(drone).unwrap();
    assert_eq!(
        action.current,
        ActionType::Explore,
        "Dumb drones should default to Explore action"
    );
}

#[test]
fn test_swarm_intelligence_smart_drone() {
    let mut app = bevy_app::App::new();
    app.add_systems(
        bevy_app::Update,
        (update_drone_clusters, update_drone_behavior).chain(),
    );

    let drone1 = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 0, y: 0 },
            DroneBehavior::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 0,
            },
        ))
        .id();

    let _drone2 = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 1, y: 0 },
            DroneBehavior::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 0,
            },
        ))
        .id();

    let _drone3 = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 0, y: 1 },
            DroneBehavior::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 0,
            },
        ))
        .id();

    app.update();

    let behavior = app.world().get::<DroneBehavior>(drone1).unwrap();
    assert_eq!(behavior.intelligence_level, IntelligenceLevel::High);

    let action = app.world().get::<PopAction>(drone1).unwrap();
    assert_eq!(
        action.current,
        ActionType::Repair,
        "Smart drones should default to Repair action"
    );
}

#[test]
fn test_swarm_intelligence_does_not_interrupt_charging() {
    let mut app = bevy_app::App::new();
    app.add_systems(
        bevy_app::Update,
        (update_drone_clusters, update_drone_behavior).chain(),
    );

    let drone = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 0, y: 0 },
            DroneBehavior {
                intelligence_level: IntelligenceLevel::High,
            },
            PopAction {
                current: ActionType::Charge,
                current_utility: 1.0,
                ticks_committed: 0,
            },
        ))
        .id();

    let _drone2 = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 1, y: 0 },
            DroneBehavior::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 0,
            },
        ))
        .id();

    let _drone3 = app
        .world_mut()
        .spawn((
            Drone::default(),
            GridPosition { x: 0, y: 1 },
            DroneBehavior::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 0,
            },
        ))
        .id();

    app.update();

    let behavior = app.world().get::<DroneBehavior>(drone).unwrap();
    assert_eq!(behavior.intelligence_level, IntelligenceLevel::High);

    let action = app.world().get::<PopAction>(drone).unwrap();
    assert_eq!(
        action.current,
        ActionType::Charge,
        "Swarm logic should not interrupt a drone that is charging"
    );
}
