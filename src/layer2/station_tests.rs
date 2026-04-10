use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::{CargoStack, FleetCargo};
use crate::layer2::station::{Station, StationType, build_station_system};
use crate::layer2::system::{Orbit, OrbitalBody};
use bevy_ecs::prelude::*;
use ratatui::style::Color;

fn setup_world() -> World {
    let world = World::new();
    // Register components
    world
}

#[test]
fn test_station_component() {
    let mut world = setup_world();
    let station = world
        .spawn((
            Station {
                station_type: StationType::Outpost,
                gravity: crate::layer2::station::GravityLevel::ZeroG,
            },
            OrbitalBody {
                name: "Alpha Station".to_string(),
                radius: 1.0,
                color: Color::Gray,
                char: '+',
            },
        ))
        .id();

    let s = world.get::<Station>(station).unwrap();
    assert_eq!(s.station_type, StationType::Outpost);
}

#[test]
fn test_build_station_consumes_resources() {
    let mut world = setup_world();
    let planet = world.spawn_empty().id();

    // Setup Fleet with resources
    let fleet = world
        .spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                capacity: 100.0,
            },
        ))
        .id();

    // Issue Build Order (Cost: 50 Metal)
    world
        .entity_mut(fleet)
        .insert(FleetOrder::BuildStation(StationType::Outpost));

    // Run Build System
    let mut schedule = Schedule::default();
    schedule.add_systems(build_station_system);
    schedule.run(&mut world);

    // Verify Resources Deducted
    let cargo = world.get::<FleetCargo>(fleet).unwrap();
    assert_eq!(cargo.contents[0].amount, 50.0);

    // Verify Order Consumed
    assert!(world.get::<FleetOrder>(fleet).is_none());
}

#[test]
fn test_build_station_spawns_entity() {
    let mut world = setup_world();
    let planet = world.spawn_empty().id();

    let fleet = world
        .spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                capacity: 100.0,
            },
        ))
        .id();

    world
        .entity_mut(fleet)
        .insert(FleetOrder::BuildStation(StationType::Outpost));

    let mut schedule = Schedule::default();
    schedule.add_systems(build_station_system);
    schedule.run(&mut world);

    // Verify Station Spawned
    let station_count = world.query::<&Station>().iter(&world).count();
    assert_eq!(station_count, 1);

    let (_entity, station, orbit) = world.query::<(Entity, &Station, &Orbit)>().single(&world);
    assert_eq!(station.station_type, StationType::Outpost);
    assert_eq!(orbit.parent, planet);
}

#[test]
fn test_build_station_fails_insufficient_resources() {
    let mut world = setup_world();
    let planet = world.spawn_empty().id();

    let fleet = world
        .spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 10.0, // Need 50
                }],
                capacity: 100.0,
            },
        ))
        .id();

    world
        .entity_mut(fleet)
        .insert(FleetOrder::BuildStation(StationType::Outpost));

    let mut schedule = Schedule::default();
    schedule.add_systems(build_station_system);
    schedule.run(&mut world);

    // Verify No Station
    let station_count = world.query::<&Station>().iter(&world).count();
    assert_eq!(station_count, 0);

    // Verify Order Not Consumed
    assert!(world.get::<FleetOrder>(fleet).is_some());
}
