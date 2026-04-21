use bevy::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::control::{DoorControl, DoorState};
use scale::layer1::health::Health;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::{Pop, Speed};
use scale::layer1::pressure::{
    apply_door_movement_penalties_system, pressure_damage_system, process_door_venting_system,
    PressureGrid,
};

#[test]
fn test_door_venting_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, process_door_venting_system);

    app.world_mut().insert_resource(PressureGrid::new(10, 10));
    app.world_mut()
        .resource_mut::<PressureGrid>()
        .set(4, 5, 1.0);
    app.world_mut()
        .resource_mut::<PressureGrid>()
        .set(6, 5, 0.0);

    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Airlock,
        },
        GridPosition { x: 5, y: 5 },
        DoorControl {
            state: DoorState::Open,
        },
    ));

    app.update();

    let grid = app.world().resource::<PressureGrid>();
    assert!(
        grid.get(4, 5) < 1.0,
        "Pressure should vent from interior ({} < 1.0)",
        grid.get(4, 5)
    );
    assert!(
        grid.get(6, 5) > 0.0,
        "Pressure should vent to exterior ({} > 0.0)",
        grid.get(6, 5)
    );
}

#[test]
fn test_airlock_movement_penalty_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, apply_door_movement_penalties_system);

    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Airlock,
        },
        GridPosition { x: 2, y: 2 },
    ));

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 2, y: 2 },
            Speed {
                base: 10.0,
                current: 10.0,
                accumulator: 0.0,
            },
        ))
        .id();

    app.update();

    let speed = app.world().get::<Speed>(pop_entity).unwrap();
    assert!(
        speed.current < speed.base,
        "Airlock should reduce pop movement speed ({} < {})",
        speed.current,
        speed.base
    );
}

#[test]
fn test_pressure_damage_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, pressure_damage_system);
    app.world_mut().init_resource::<Events<AddChronicleEvent>>();

    app.world_mut().insert_resource(PressureGrid::new(10, 10));
    // Tile 2,2 has 0.0 pressure (vacuum)

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 2, y: 2 },
            Health {
                current: 100.0,
                max: 100.0,
                conditions: Vec::new(),
            },
        ))
        .id();

    app.update();

    let health = app.world().get::<Health>(pop_entity).unwrap();
    assert!(
        health.current < 100.0,
        "Pop should take damage in vacuum ({} < 100.0)",
        health.current
    );
}
