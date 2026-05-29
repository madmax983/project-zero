use bevy::prelude::*;
use scale::layer2::fleet::MovementSpeed;
use scale::layer2::fleet::{fleet_movement_system, Fleet, InTransit};
use scale::layer2::gravitational_doldrums::{
    doldrums_effects_system, GravitationalDoldrums, TugShip,
};
use scale::layer2::nebulae::SpatialVolume;

#[test]
fn test_doldrums_integration_movement() {
    let mut app = App::new();

    // The order should be doldrums_effects_system then fleet_movement_system
    app.add_systems(
        Update,
        (doldrums_effects_system, fleet_movement_system).chain(),
    );

    let planet_a = app.world_mut().spawn_empty().id();
    let planet_b = app.world_mut().spawn_empty().id();

    // Spawn a doldrums region
    app.world_mut().spawn((
        GravitationalDoldrums {
            penalty_multiplier: 0.1,
        },
        SpatialVolume { radius: 10.0 },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn a fleet inside the doldrums
    let normal_fleet = app
        .world_mut()
        .spawn((
            Fleet,
            Transform::from_translation(Vec3::ZERO),
            MovementSpeed {
                base: 10.0,
                current: 10.0,
            },
            InTransit {
                origin: planet_a,
                destination: planet_b,
                progress: 0.0,
                duration: 100.0, // 100 ticks at 1.0 speed
            },
        ))
        .id();

    let tug_fleet = app
        .world_mut()
        .spawn((
            Fleet,
            Transform::from_translation(Vec3::ZERO),
            TugShip { tow_capacity: 10 },
            MovementSpeed {
                base: 10.0,
                current: 10.0,
            },
            InTransit {
                origin: planet_a,
                destination: planet_b,
                progress: 0.0,
                duration: 100.0, // 100 ticks at 1.0 speed
            },
        ))
        .id();

    app.update();

    let normal_transit = app.world().get::<InTransit>(normal_fleet).unwrap();
    let normal_speed = app.world().get::<MovementSpeed>(normal_fleet).unwrap();

    let tug_transit = app.world().get::<InTransit>(tug_fleet).unwrap();
    let tug_speed = app.world().get::<MovementSpeed>(tug_fleet).unwrap();

    assert_eq!(normal_speed.current, 1.0);
    assert!((normal_transit.progress - 0.001).abs() < f32::EPSILON);
    assert_eq!(tug_speed.current, 10.0);
    assert!((tug_transit.progress - 0.01).abs() < f32::EPSILON);
}
