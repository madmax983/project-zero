use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::health::{check_health_status_system, Health};
use scale::layer1::map::{GridPosition, ScreenShake};
use scale::layer1::pop::{handle_pop_death_system, Pop, PopDied, PopName};
use scale::layer1::pressure::{pressure_damage_system, update_pressure_system, PressureGrid};
use scale::layer1::GlobalHitStop;
use scale::setup::{setup_world, setup_world_with_config, SetupConfig};
use scale::shared::log::MessageLog;

const STARTER_HAZARD_BUFFER_RADIUS: i32 = 12;

fn squared_distance(a: GridPosition, b: GridPosition) -> i32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

#[test]
fn test_setup_world_bootstraps_breathable_start_for_initial_pops() {
    let mut world = setup_world();

    let lander_count = world
        .query::<&Building>()
        .iter(&world)
        .filter(|building| building.building_type == BuildingType::Lander)
        .count();
    assert!(
        lander_count > 0,
        "A new colony should start with a Lander building"
    );

    for _ in 0..10 {
        world.run_system_once(update_pressure_system).unwrap();
    }

    let pop_positions: Vec<GridPosition> = world
        .query::<(&Pop, &GridPosition)>()
        .iter(&world)
        .map(|(_, pos)| *pos)
        .collect();
    assert_eq!(pop_positions.len(), 5, "Expected 5 initial pops");

    let pressure = world.resource::<PressureGrid>();
    for pos in pop_positions {
        let tile_pressure = pressure.get(pos.x, pos.y);
        assert!(
            tile_pressure >= 0.2,
            "Initial pop at ({}, {}) should start in breathable pressure, got {}",
            pos.x,
            pos.y,
            tile_pressure
        );
    }
}

#[test]
fn test_ancient_structures_spawn_outside_starter_habitat_buffer() {
    let mut world = setup_world_with_config(SetupConfig {
        headless: true,
        ..Default::default()
    });

    let lander_pos = world
        .query::<(&Building, &GridPosition)>()
        .iter(&world)
        .find(|(building, _)| building.building_type == BuildingType::Lander)
        .map(|(_, pos)| *pos)
        .expect("Starter colony should include a Lander");

    let ancient_structures: Vec<_> = world
        .query::<(&Building, &GridPosition)>()
        .iter(&world)
        .filter(|(building, _)| {
            matches!(
                building.building_type,
                BuildingType::AncientReactor | BuildingType::AncientFabricator
            )
        })
        .map(|(building, pos)| (building.building_type, *pos))
        .collect();

    assert_eq!(
        ancient_structures.len(),
        2,
        "Expected both ancient starter structures to spawn"
    );

    for (building_type, pos) in ancient_structures {
        assert!(
            squared_distance(lander_pos, pos)
                >= STARTER_HAZARD_BUFFER_RADIUS * STARTER_HAZARD_BUFFER_RADIUS,
            "{building_type:?} spawned too close to the starter habitat at ({}, {})",
            pos.x,
            pos.y
        );
    }
}

#[test]
fn test_suffocation_death_reports_atmospheric_breach_reason() {
    let mut world = World::new();
    world.insert_resource(PressureGrid::new(5, 5));
    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<Events<PopDied>>();

    world.spawn((
        Pop,
        PopName("Ada".to_string()),
        GridPosition { x: 2, y: 2 },
        Health {
            current: 1.0,
            max: 100.0,
            has_rust_lung: false,
        },
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((
        pressure_damage_system,
        check_health_status_system.after(pressure_damage_system),
        handle_pop_death_system.after(check_health_status_system),
    ));
    schedule.run(&mut world);

    let events = world.resource::<Events<PopDied>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Expected one pop death event");
    assert_eq!(emitted[0].reason, "Atmospheric breach");
}

#[test]
fn test_pop_death_feedback_is_not_camera_burnout() {
    let mut world = World::new();
    world.insert_resource(ScreenShake::default());
    world.insert_resource(GlobalHitStop::default());
    world.insert_resource(MessageLog::default());
    world.init_resource::<Events<PopDied>>();

    world.spawn((
        Pop,
        PopName("Tarn".to_string()),
        GridPosition { x: 2, y: 2 },
        Health {
            current: 0.0,
            max: 100.0,
            has_rust_lung: false,
        },
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((
        check_health_status_system,
        handle_pop_death_system.after(check_health_status_system),
    ));
    schedule.run(&mut world);

    let shake = world.resource::<ScreenShake>();
    assert!(
        shake.intensity <= 0.25,
        "A single pop death should not induce a huge camera shake, got {}",
        shake.intensity
    );

    let hit_stop = world.resource::<GlobalHitStop>();
    assert!(
        hit_stop.ticks <= 2,
        "A single pop death should not freeze the game for long, got {} ticks",
        hit_stop.ticks
    );
}
