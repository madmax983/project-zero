use bevy_ecs::prelude::*;
use scale::layer1::integration::fleet_unload_system;
use scale::layer1::resources::ColonyResources;
use scale::layer1::resources::ResourceType;
use scale::layer2::fleet::{Fleet, InOrbit};
use scale::layer2::generation::ColonyLocation;
use scale::layer2::mining::{CargoStack, FleetCargo};
use scale::shared::log::MessageLog;

#[test]
fn test_fleet_unloads_cargo_at_colony() {
    let mut world = World::new();

    // Resources
    world.insert_resource(ColonyResources::default());
    world.insert_resource(MessageLog::default());

    // Setup:
    // 1. Colony Location
    let colony_planet = world.spawn(ColonyLocation).id();

    // 2. Fleet with Cargo in Orbit
    let fleet = world
        .spawn((
            Fleet,
            InOrbit {
                parent: colony_planet,
            },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 10.0,
                }],
                capacity: 100.0,
            },
        ))
        .id();

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(fleet_unload_system);
    schedule.run(&mut world);

    // Assert: Colony has Ore
    let resources = world.resource::<ColonyResources>();
    assert!(
        (resources.ore - 10.0).abs() < f32::EPSILON,
        "Colony should receive 10.0 Ore, but has {}",
        resources.ore
    );

    // Assert: Fleet is empty
    let cargo = world.get::<FleetCargo>(fleet).unwrap();
    assert!(
        cargo.current_load() < f32::EPSILON,
        "Fleet cargo should be empty, but has {}",
        cargo.current_load()
    );
}

#[test]
fn test_fleet_does_not_unload_elsewhere() {
    let mut world = World::new();

    world.insert_resource(ColonyResources::default());
    world.insert_resource(MessageLog::default());

    // Setup:
    // 1. Some other planet (NOT ColonyLocation)
    let random_planet = world.spawn_empty().id();
    // 2. Colony Location (somewhere else)
    let _colony_planet = world.spawn(ColonyLocation).id();

    // 3. Fleet with Cargo at random planet
    let fleet = world
        .spawn((
            Fleet,
            InOrbit {
                parent: random_planet,
            },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 10.0,
                }],
                capacity: 100.0,
            },
        ))
        .id();

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(fleet_unload_system);
    schedule.run(&mut world);

    // Assert: Colony has NO Ore
    let resources = world.resource::<ColonyResources>();
    assert!(
        resources.ore < f32::EPSILON,
        "Colony should not receive Ore from remote fleet"
    );

    // Assert: Fleet still has cargo
    let cargo = world.get::<FleetCargo>(fleet).unwrap();
    assert!(
        (cargo.current_load() - 10.0).abs() < f32::EPSILON,
        "Fleet should keep cargo"
    );
}
