use bevy::prelude::*;
use scale::layer1::biology::health::Health;
use scale::layer1::core::integration::{localized_gravity_impact_damage_bridge, HasFallen};
use scale::layer1::entities::pop::Pop;
use scale::layer1::physics::gravity_plating::{GravityPlate};

#[test]
fn test_localized_gravity_health_bridge_damage() {
    let mut app = App::new();

    // The bridge checks if there's any powered plate.
    // If not, it applies damage once.

    app.add_systems(Update, localized_gravity_impact_damage_bridge);

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
        ))
        .id();

    let _unpowered_plate = app
        .world_mut()
        .spawn(GravityPlate {
            down_vector: Vec3::Y,
            powered: false,
        })
        .id();

    app.update();

    let health = app.world().get::<Health>(pop).unwrap();
    assert_eq!(
        health.current, 50.0,
        "Pop should take 50 damage upon falling due to unpowered plates."
    );
    assert!(
        app.world().get::<HasFallen>(pop).is_some(),
        "Pop should have HasFallen marker added."
    );

    // Update again to verify damage is not applied multiple times
    app.update();
    let health2 = app.world().get::<Health>(pop).unwrap();
    assert_eq!(
        health2.current, 50.0,
        "Pop should not take continuous damage."
    );
}

#[test]
fn test_localized_gravity_health_bridge_recovery() {
    let mut app = App::new();
    app.add_systems(Update, localized_gravity_impact_damage_bridge);

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Health {
                current: 50.0,
                max: 100.0,
                has_rust_lung: false,
            },
            HasFallen,
        ))
        .id();

    let _powered_plate = app
        .world_mut()
        .spawn(GravityPlate {
            down_vector: Vec3::Y,
            powered: true,
        })
        .id();

    app.update();

    assert!(
        app.world().get::<HasFallen>(pop).is_none(),
        "HasFallen marker should be removed when gravity is restored."
    );
}
