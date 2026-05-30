use bevy::prelude::*;
use scale::layer2::generation::Planet;
use scale::layer2::planetary_spin_up::{
    apply_planetary_torque_system, calculate_effective_gravity_system,
    trigger_coriolis_weather_system, AttachedToPlanet, BaseGravity, DayLength, EffectiveGravity,
    PlanetaryEngine, PlanetaryTorqueEvent, WeatherState,
};

#[test]
fn test_planetary_spin_up_integration() {
    let mut app = App::new();

    app.add_event::<PlanetaryTorqueEvent>();
    app.add_systems(
        Update,
        (
            apply_planetary_torque_system,
            calculate_effective_gravity_system,
            trigger_coriolis_weather_system,
        )
            .chain(),
    );

    let planet = app
        .world_mut()
        .spawn((
            Planet,
            DayLength { hours: 24.0 },
            BaseGravity { g: 1.0 },
            EffectiveGravity { g: 1.0 },
            WeatherState::Calm,
        ))
        .id();

    app.world_mut().spawn((
        PlanetaryEngine {
            torque: 10.0,
            active: true,
        },
        AttachedToPlanet(planet),
    ));

    app.update();

    let day_length = app.world().get::<DayLength>(planet).unwrap();
    assert!(
        day_length.hours < 24.0,
        "Day length should decrease due to torque"
    );

    let gravity = app.world().get::<EffectiveGravity>(planet).unwrap();
    assert!(
        gravity.g < 1.0,
        "Effective gravity should decrease due to spin"
    );

    let weather = app.world().get::<WeatherState>(planet).unwrap();
    assert_eq!(
        *weather,
        WeatherState::Chaotic,
        "Weather should become chaotic from high torque"
    );
}
