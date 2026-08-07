use scale::layer1::environment::orbital_junkyard::DebrisRainChance;
use scale::layer2::debris::OrbitalDebris;
use scale::layer2::integration::orbital_junkyard_bridge_system;
use bevy::prelude::*;

#[test]
fn test_orbital_junkyard_bridge_system() {
    let mut app = App::new();
    app.add_systems(Update, orbital_junkyard_bridge_system);
    app.insert_resource(DebrisRainChance(0.0));

    // Initially no debris
    app.update();
    assert_eq!(app.world().resource::<DebrisRainChance>().0, 0.0);

    // Spawn some orbital debris
    app.world_mut().spawn(OrbitalDebris(5.0));
    app.world_mut().spawn(OrbitalDebris(2.0));

    app.update();

    // (5.0 + 2.0) * 0.1 = 0.7
    let chance = app.world().resource::<DebrisRainChance>().0;
    assert!((chance - 0.7).abs() < f32::EPSILON);

    // Test clamping
    app.world_mut().spawn(OrbitalDebris(10.0));
    app.update();

    let clamped_chance = app.world().resource::<DebrisRainChance>().0;
    assert_eq!(clamped_chance, 1.0);
}
