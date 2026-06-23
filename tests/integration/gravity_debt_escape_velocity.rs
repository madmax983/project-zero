use bevy::prelude::*;
use scale::layer2::integration::gravity_debt_to_planetary_gravity_system;
use scale::layer2::orbit::gravity_debt::{GravityDebt, Planet};
use scale::layer2::syzygy::PlanetaryGravity;

#[test]
fn test_gravity_debt_modifies_planetary_gravity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, gravity_debt_to_planetary_gravity_system);

    app.world_mut().insert_resource(PlanetaryGravity::default());

    app.world_mut().spawn((
        Planet { base_gravity: 1.0 },
        GravityDebt {
            imported_mass: 10000.0,
            exported_mass: 0.0,
        },
    ));

    app.update();

    let gravity = app.world().resource::<PlanetaryGravity>();
    assert!(
        gravity.base > 1.0,
        "High gravity debt should increase PlanetaryGravity base"
    );
    assert!(
        gravity.current > 1.0,
        "High gravity debt should increase PlanetaryGravity current"
    );
    assert_eq!(
        gravity.base, gravity.current,
        "Base and current should match unless modified by syzygy"
    );
}

#[test]
fn test_no_gravity_debt_no_change() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, gravity_debt_to_planetary_gravity_system);

    app.world_mut().insert_resource(PlanetaryGravity::default());

    app.world_mut().spawn((
        Planet { base_gravity: 1.0 },
        GravityDebt {
            imported_mass: 0.0,
            exported_mass: 0.0,
        },
    ));

    app.update();

    let gravity = app.world().resource::<PlanetaryGravity>();
    assert_eq!(gravity.base, 1.0, "No debt means no increase in gravity");
    assert_eq!(gravity.current, 1.0, "No debt means no increase in gravity");
}
