use bevy_ecs::prelude::*;
use scale::layer1::biology::health::Health;
use scale::layer1::map::GridPosition;
use scale::layer2::pulsar::{
    pulsar_radiation_damage_system, pulsar_rotation_system, Facing, Pulsar,
};
use scale::layer2::ship::{Ship, ShipType};
use scale::shared::time::SimulationTime;

#[test]
fn test_pulsar_integration_wiring() {
    let mut app = bevy_app::App::new();

    // In our tests, we will add the time, and the relevant systems
    app.insert_resource(SimulationTime {
        tick: 10,
        ..Default::default()
    });

    app.add_systems(
        bevy_app::Update,
        (pulsar_rotation_system, pulsar_radiation_damage_system).chain(),
    );

    let _pulsar_id = app
        .world_mut()
        .spawn((
            Pulsar {
                rotation_period: 10.0,
                beam_width: std::f32::consts::PI / 4.0,
            },
            Facing(0.0),
            GridPosition { x: 0, y: 0 },
        ))
        .id();

    let ship_in_beam = app
        .world_mut()
        .spawn((
            Ship::new(ShipType::Scout),
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            GridPosition { x: 10, y: 0 },
        ))
        .id();

    app.update();

    let health_in = app.world().get::<Health>(ship_in_beam).unwrap();
    assert!(health_in.current < 100.0, "Ship in beam should take damage");
}
