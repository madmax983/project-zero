use bevy::MinimalPlugins;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::environment::light_pollution::{
    apply_light_pollution_system, calculate_sky_glow_system, SkyGlow,
};
use scale::layer1::fauna::{Fauna, FaunaType, NocturnalFauna};
use scale::layer1::integration::nocturnal_aggression_bridge_system;
use scale::layer1::lighting::LightSource;
use scale::layer1::map::GridPosition;

#[test]
fn test_nocturnal_aggression_increases_detection_range() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SkyGlow>();

    app.add_systems(
        Update,
        (
            calculate_sky_glow_system,
            apply_light_pollution_system.after(calculate_sky_glow_system),
            nocturnal_aggression_bridge_system.after(apply_light_pollution_system),
        ),
    );

    let base_detection_range = 8.0;

    let fauna_entity = app
        .world_mut()
        .spawn((
            Fauna {
                fauna_type: FaunaType::SpaceRat,
                detection_range: base_detection_range,
                ..Default::default()
            },
            NocturnalFauna { aggression: 0.0 },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    // Spawn an outdoor light
    app.world_mut().spawn((
        LightSource {
            radius: 10.0,
            intensity: 10.0,
            is_outdoor: true,
            color: (255, 255, 255),
        },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    let fauna = app.world().get::<Fauna>(fauna_entity).unwrap();
    assert!(
        fauna.detection_range > base_detection_range,
        "Fauna detection range should increase due to nocturnal aggression"
    );
    assert!(
        fauna.detection_range <= 8.0 + 15.0,
        "Fauna detection range should not exceed maximum bonus"
    );
}

#[test]
fn test_nocturnal_aggression_does_not_grow_infinitely() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SkyGlow>();

    app.add_systems(
        Update,
        (
            calculate_sky_glow_system,
            apply_light_pollution_system.after(calculate_sky_glow_system),
            nocturnal_aggression_bridge_system.after(apply_light_pollution_system),
        ),
    );

    let base_detection_range = 8.0;

    let fauna_entity = app
        .world_mut()
        .spawn((
            Fauna {
                fauna_type: FaunaType::SpaceRat,
                detection_range: base_detection_range,
                ..Default::default()
            },
            NocturnalFauna { aggression: 0.0 },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    // Spawn an outdoor light
    app.world_mut().spawn((
        LightSource {
            radius: 10.0,
            intensity: 10.0,
            is_outdoor: true,
            color: (255, 255, 255),
        },
        GridPosition { x: 5, y: 5 },
    ));

    for _ in 0..1000 {
        app.update();
    }

    let fauna = app.world().get::<Fauna>(fauna_entity).unwrap();

    // Aggression will be very high, so it should be capped
    assert!(
        fauna.detection_range <= base_detection_range + 15.0 + 0.2, // Some floating point tolerance
        "Fauna detection range should be capped at max bonus: {}",
        fauna.detection_range
    );
}
