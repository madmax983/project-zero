use bevy::prelude::*;
use scale::layer1::environment::terminator_habitats::{
    apply_libration_wobble_system, building_temperature_damage_system,
    calculate_tile_temperatures_system, Building, Health, LibrationCycle, LocatedOn,
    MaxTemperatureAllowed, Temperature, TerminatorLine, Tile,
};

#[test]
fn test_terminator_habitat_integration() {
    let mut app = App::new();

    app.add_systems(
        Update,
        (
            apply_libration_wobble_system,
            calculate_tile_temperatures_system,
            building_temperature_damage_system,
        )
            .chain(),
    );

    app.world_mut()
        .insert_resource(TerminatorLine { x_coordinate: 50.0 });
    app.world_mut().insert_resource(LibrationCycle {
        current_tick: 0.0,
        amplitude: 10.0,
        speed: std::f32::consts::PI / 2.0, // Moves sin() from 0 to 1
    });

    let tile = app
        .world_mut()
        .spawn((Tile { x: 55.0, y: 10.0 }, Temperature { degrees: 0.0 }))
        .id();

    let building = app
        .world_mut()
        .spawn((
            Building,
            Health {
                current: 100.0,
                max: 100.0,
            },
            MaxTemperatureAllowed { degrees: 40.0 }, // Can't survive Day side (150.0)
            LocatedOn(tile),
        ))
        .id();

    // Tick 1: Libration shifts Terminator to 60.0.
    // Tile at 55 is now < 60, distance = -5.0 (Day side!)
    app.update();

    let health = app.world().get::<Health>(building).unwrap();
    assert!(
        health.current < 100.0,
        "Building should take damage after being exposed to Day side by libration"
    );
}
