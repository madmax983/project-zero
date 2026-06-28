use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::{CargoStack, FleetCargo};
use crate::layer2::station::{build_station_system, Station, StationType};
use crate::layer2::system::{Orbit, OrbitalBody};
use bevy::prelude::*;
use rand::SeedableRng;

use ratatui::style::Color;

fn setup_world() -> World {
    World::new()
}

#[test]
fn test_station_component() {
    let mut world = setup_world();
    let station = world
        .spawn((
            Station {
                station_type: StationType::Outpost,
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

#[test]
fn test_zero_g_fermentation_production() {
    let mut world = setup_world();
    let planet = world.spawn_empty().id();

    // Arrange: Setup world with an orbital station capable of fermentation
    let orbital_station = world
        .spawn((
            Station {
                station_type: StationType::Brewery,
            },
            OrbitalBody {
                name: "Brewery Station".to_string(),
                radius: 1.0,
                color: Color::Gray,
                char: 'B',
            },
            Orbit {
                parent: planet,
                radius: 10.0,
                speed: 0.1,
                angle: 0.0,
            },
            crate::layer2::system::GravityLevel::ZeroG,
            crate::layer2::station::ZeroGBrewery {
                production_time: bevy_time::Timer::new(
                    std::time::Duration::from_secs_f32(1.0),
                    bevy_time::TimerMode::Repeating,
                ),
            },
            crate::layer1::economy::inventory::Inventory::default(),
        ))
        .id();

    // Act: Advance simulation time for a production cycle
    let mut schedule = Schedule::default();
    schedule.add_systems(crate::layer2::station::zero_g_fermentation_system);

    let mut time: bevy_time::Time = bevy_time::Time::default();
    time.advance_by(std::time::Duration::from_secs_f32(1.1));
    world.insert_resource(time);

    schedule.run(&mut world);

    // Assert: Verify Void-Ale was produced in the station's inventory
    let inventory = world
        .get::<crate::layer1::economy::inventory::Inventory>(orbital_station)
        .unwrap();
    let has_ale = inventory
        .items
        .iter()
        .any(|item| item.item_type == crate::layer1::items::ItemType::VoidAle);
    assert!(has_ale, "Zero-G fermentation should produce Void-Ale");
}

#[test]
fn test_zero_g_fermentation_consumption_morale() {
    let mut world = setup_world();

    // Arrange: Setup world with a colonist consuming Void-Ale
    let pop_entity = world
        .spawn((
            crate::layer1::pop::Pop,
            crate::layer1::needs::Needs {
                leisure: 0.2, // Low leisure
                ..Default::default()
            },
            crate::layer1::economy::inventory::Inventory {
                items: vec![crate::layer1::economy::inventory::InventoryItem {
                    item_type: crate::layer1::items::ItemType::VoidAle,
                    entity: None,
                }],
                capacity: 5,
            },
        ))
        .id();

    // Act: Advance simulation to trigger consumption
    let initial_morale = world
        .get::<crate::layer1::needs::Needs>(pop_entity)
        .unwrap()
        .leisure;

    let mut schedule = Schedule::default();
    schedule.add_systems(crate::layer1::systems::consumption::consume_void_ale_system);
    schedule.run(&mut world);

    // Assert: Verify morale increases significantly
    let new_morale = world
        .get::<crate::layer1::needs::Needs>(pop_entity)
        .unwrap()
        .leisure;
    assert!(
        new_morale > initial_morale,
        "Consuming Void-Ale should increase morale"
    );

    let inv = world
        .get::<crate::layer1::economy::inventory::Inventory>(pop_entity)
        .unwrap();
    assert!(inv.items.is_empty(), "Void-Ale should be consumed");
}

#[test]
fn test_zero_g_fermentation_requires_zero_g() {
    let mut world = setup_world();
    let planet = world.spawn_empty().id();

    // Arrange: Attempt to produce Void-Ale on the ground (MicroGravity or Normal)
    let ground_brewery = world
        .spawn((
            Station {
                station_type: StationType::Brewery,
            },
            OrbitalBody {
                name: "Ground Brewery".to_string(),
                radius: 1.0,
                color: Color::Gray,
                char: 'B',
            },
            Orbit {
                parent: planet,
                radius: 1.0,
                speed: 0.1,
                angle: 0.0,
            },
            crate::layer2::system::GravityLevel::Normal,
            crate::layer2::station::ZeroGBrewery {
                production_time: bevy_time::Timer::new(
                    std::time::Duration::from_secs_f32(1.0),
                    bevy_time::TimerMode::Repeating,
                ),
            },
            crate::layer1::economy::inventory::Inventory::default(),
        ))
        .id();

    // Act: Advance simulation time
    let mut schedule = Schedule::default();
    schedule.add_systems(crate::layer2::station::zero_g_fermentation_system);

    let mut time: bevy_time::Time = bevy_time::Time::default();
    time.advance_by(std::time::Duration::from_secs_f32(1.1));
    world.insert_resource(time);

    schedule.run(&mut world);

    // Assert: Verify Void-Ale was NOT produced
    let inventory = world
        .get::<crate::layer1::economy::inventory::Inventory>(ground_brewery)
        .unwrap();
    let has_ale = inventory
        .items
        .iter()
        .any(|item| item.item_type == crate::layer1::items::ItemType::VoidAle);
    assert!(!has_ale, "Void-Ale cannot be produced on the ground");
}

#[test]
fn test_deep_forge_production() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(crate::layer1::economy::resources::ColonyResources::default());
    app.world_mut()
        .insert_resource(crate::shared::random::GlobalRng(
            rand::rngs::StdRng::seed_from_u64(0),
        ));
    app.add_event::<crate::layer2::station::ForgeCrushEvent>();
    app.add_systems(Update, crate::layer2::station::process_deep_forges);

    let forge = app
        .world_mut()
        .spawn((
            crate::layer2::station::DeepForge {
                production_rate: 10.0,
                base_crush_chance: 0.0,
                is_active: true,
            },
            crate::layer2::station::MaintenanceLevel { current: 100.0 }, // Perfect maintenance
        ))
        .id();

    // Act
    app.update();

    // Assert
    let resources = app
        .world()
        .get_resource::<crate::layer1::economy::resources::ColonyResources>()
        .unwrap();
    assert_eq!(
        resources.hyper_alloys, 10.0,
        "Active forge should produce hyper-alloys"
    );

    let forge_exists = app
        .world()
        .get::<crate::layer2::station::DeepForge>(forge)
        .is_some();
    assert!(forge_exists, "Perfectly maintained forge should survive");
}

#[test]
fn test_deep_forge_crush_failure() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(crate::layer1::economy::resources::ColonyResources::default());
    app.world_mut()
        .insert_resource(crate::shared::random::GlobalRng(
            rand::rngs::StdRng::seed_from_u64(0),
        ));
    app.add_event::<crate::layer2::station::ForgeCrushEvent>();
    app.add_systems(Update, crate::layer2::station::process_deep_forges);

    let forge = app
        .world_mut()
        .spawn((
            crate::layer2::station::DeepForge {
                production_rate: 10.0,
                base_crush_chance: 1.0, // Guaranteed failure
                is_active: true,
            },
            crate::layer2::station::MaintenanceLevel { current: 0.0 }, // Zero maintenance
            crate::layer2::station::Crew { count: 50 },
        ))
        .id();

    // Act
    app.update();

    // Assert
    let forge_exists = app
        .world()
        .get::<crate::layer2::station::DeepForge>(forge)
        .is_some();
    assert!(!forge_exists, "Poorly maintained forge should be crushed");

    // Crew should be killed, triggering a chronicle event
    let crush_events = app
        .world()
        .resource::<Events<crate::layer2::station::ForgeCrushEvent>>();
    let reader = crush_events.get_cursor();
    assert!(
        reader.len(crush_events) > 0,
        "A crush event should be spawned"
    );
}
