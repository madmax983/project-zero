use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::psychic::apply_psychic_radiation_system;
use scale::layer1::stress::StressTracker;
use scale::layer2::environment::PsychicBackground;
use scale::shared::time::SimulationTime;
use bevy_time::Time;

#[test]
fn test_integration_psychic_psychology() {
    let mut app = App::new();

    // Init Resources
    app.insert_resource(PsychicBackground { intensity: 0.8 });

    let mut time: Time = Time::default();
    time.advance_by(std::time::Duration::from_secs(1));
    app.insert_resource(time);

    // Add Systems
    app.add_systems(bevy_app::Update, apply_psychic_radiation_system);

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                rest: 100.0,
                ..Default::default()
            },
            StressTracker {
                accumulated_stress: 0.0,
                ..Default::default()
            },
        ))
        .id();

    app.update();

    let needs = app.world().get::<Needs>(pop_entity).unwrap();
    let stress = app.world().get::<StressTracker>(pop_entity).unwrap();

    assert!(
        needs.rest < 100.0,
        "Needs.rest should decrease due to psychic radiation"
    );
    assert!(
        stress.accumulated_stress > 0.0,
        "StressTracker.accumulated_stress should increase due to psychic radiation"
    );
}
