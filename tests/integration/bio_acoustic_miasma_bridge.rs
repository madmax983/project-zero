use bevy::prelude::*;
use scale::layer1::environment::bio_acoustic_miasma::ParanoiaTracker;
use scale::layer1::integration::paranoia_stress_bridge_system;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::stress::{check_stress_breakdown_system, Breakdown, StressTracker};
use scale::layer1::traits::Traits;

#[test]
fn test_paranoia_triggers_stress_breakdown() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register systems
    app.add_systems(
        Update,
        (paranoia_stress_bridge_system, check_stress_breakdown_system).chain(),
    );

    // Spawn a Pop with high paranoia but zero initial stress
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 1.0,
            },
            StressTracker {
                accumulated_stress: 0.0,
            },
            Traits::default(),
            ParanoiaTracker { level: 150 }, // Above BREAKDOWN_TICKS_REQUIRED (100.0)
        ))
        .id();

    // Run the schedule
    app.update();

    // Assert that the paranoia level was reset
    let paranoia = app.world().get::<ParanoiaTracker>(pop).unwrap();
    assert_eq!(paranoia.level, 0, "Paranoia level should be reset to 0");

    // Assert that a Breakdown was triggered due to the added paranoia
    assert!(
        app.world().get::<Breakdown>(pop).is_some(),
        "Pop should have triggered a breakdown"
    );

    // Assert that accumulated_stress was reset after breakdown
    let stress = app.world().get::<StressTracker>(pop).unwrap();
    assert_eq!(
        stress.accumulated_stress, 0.0,
        "Stress should be reset after breakdown"
    );
}

#[test]
fn test_paranoia_adds_to_stress_without_breakdown() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, paranoia_stress_bridge_system);

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            StressTracker {
                accumulated_stress: 10.0,
            },
            ParanoiaTracker { level: 20 }, // 10 + 20 = 30 < 100
        ))
        .id();

    app.update();

    let stress = app.world().get::<StressTracker>(pop).unwrap();
    assert_eq!(
        stress.accumulated_stress, 30.0,
        "Stress should accumulate paranoia"
    );

    let paranoia = app.world().get::<ParanoiaTracker>(pop).unwrap();
    assert_eq!(paranoia.level, 0, "Paranoia level should be reset");
}
