use bevy::prelude::*;
use scale::layer1::architecture::housing::Housing;
use scale::layer1::core::integration::{
    apply_ruin_environmental_buffs_system, apply_ruin_psychological_stress_system,
    trigger_ruin_machinery_system, AncientRuin, MachineryTrigger, TemperatureRegulation,
};
use scale::layer1::environment::Temperature;
use scale::layer1::pop::Pop;
use scale::layer1::psychology::stress::StressTracker;
use scale::shared::time::SimulationTime;

// Red Phase Test Setup
fn setup_app() -> App {
    let mut app = App::new();
    // Add systems
    app.add_systems(
        Update,
        (
            apply_ruin_environmental_buffs_system,
            trigger_ruin_machinery_system,
            apply_ruin_psychological_stress_system,
        ),
    );
    app.init_resource::<SimulationTime>();
    app
}

#[test]
fn test_ruin_housing_provides_temperature_buff() {
    let mut app = setup_app();

    let pop = app
        .world_mut()
        .spawn((Pop, Temperature { degrees: 10.0 }))
        .id();

    let _ruin = app
        .world_mut()
        .spawn((
            Housing {
                capacity: 1,
                residents: vec![pop],
            },
            AncientRuin { is_active: false },
            TemperatureRegulation { bonus: 20.0 },
        ))
        .id();

    app.update();

    let temp = app.world().get::<Temperature>(pop).unwrap();
    // Expecting the temp to be boosted by the ruin's regulation bonus
    assert_eq!(
        temp.degrees, 30.0,
        "Pop temperature should be buffed by ruin housing"
    );
}

#[test]
fn test_ruin_machinery_activates_randomly() {
    let mut app = setup_app();
    app.world_mut().resource_mut::<SimulationTime>().tick = 100; // Force trigger condition for test

    let ruin = app
        .world_mut()
        .spawn((
            AncientRuin { is_active: false },
            MachineryTrigger { threshold: 50 },
        ))
        .id();

    // To prevent flaky tests due to RNG, run it multiple times until it activates
    for _ in 0..100 {
        app.update();
        let ruin_data = app.world().get::<AncientRuin>(ruin).unwrap();
        if ruin_data.is_active {
            return;
        }
    }

    panic!("Ruin machinery failed to activate after 100 updates");
}

#[test]
fn test_active_ruin_causes_psychological_stress() {
    let mut app = setup_app();

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            StressTracker {
                accumulated_stress: 0.0,
            },
        ))
        .id();

    let _active_ruin = app
        .world_mut()
        .spawn((
            Housing {
                capacity: 1,
                residents: vec![pop],
            },
            AncientRuin { is_active: true },
        ))
        .id();

    app.update();

    let stress = app.world().get::<StressTracker>(pop).unwrap();
    assert!(
        stress.accumulated_stress > 0.0,
        "Pop should gain stress when housed in an active ruin"
    );
}
