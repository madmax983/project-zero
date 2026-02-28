use super::*;
use crate::layer1::health::Scars;
use crate::layer1::lifecycle::Age;
use crate::layer1::pop::Pop;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[test]
fn test_drift_accumulation_over_time() {
    let mut world = World::new();
    world.insert_resource(SimulationTime {
        tick: 1000,
        speed: crate::shared::time::SimSpeed::Normal,
    });

    let pop = world
        .spawn((
            Pop,
            BiometricProfile {
                last_update_tick: 0,
                drift: 0.0,
                recorded_scars: 0,
            },
            Age {
                ticks_alive: 5000,
                stage: crate::layer1::lifecycle::LifeStage::Adult,
            },
        ))
        .id();

    // Run drift system
    let mut schedule = Schedule::default();
    schedule.add_systems(drift_accumulation_system);
    schedule.run(&mut world);

    let profile = world.get::<BiometricProfile>(pop).unwrap();
    // 1000 ticks should cause some drift
    assert!(
        profile.drift > 0.0,
        "Drift should be greater than 0 after 1000 ticks, got {}",
        profile.drift
    );
}

#[test]
fn test_scars_increase_drift() {
    let mut world = World::new();
    world.insert_resource(SimulationTime {
        tick: 0,
        speed: crate::shared::time::SimSpeed::Normal,
    });

    let pop = world
        .spawn((
            Pop,
            BiometricProfile {
                drift: 0.0,
                last_update_tick: 0,
                recorded_scars: 0,
            },
            Scars { count: 0 },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(drift_accumulation_system);

    // Initial run (0 drift)
    schedule.run(&mut world);
    let profile = world.get::<BiometricProfile>(pop).unwrap();
    assert_eq!(profile.drift, 0.0);

    // Simulate trauma (increase scars)
    if let Some(mut scars) = world.get_mut::<Scars>(pop) {
        scars.count = 2;
    }

    // Run system again
    schedule.run(&mut world);

    let profile = world.get::<BiometricProfile>(pop).unwrap();
    // 2 scars * 0.1 drift per scar = 0.2
    assert!(
        (profile.drift - 0.2).abs() < f32::EPSILON,
        "Drift should be 0.2 from 2 scars, got {}",
        profile.drift
    );
}

#[test]
fn test_access_denied_high_drift() {
    let mut world = World::new();
    let pop = world
        .spawn((
            Pop,
            BiometricProfile {
                drift: 0.9,
                ..Default::default()
            }, // 90% Drift
        ))
        .id();

    let terminal = world
        .spawn(SecurityTerminal {
            required_clearance: 1,
            strictness: 1.0, // Strict
        })
        .id();

    // Check Access
    let result = check_access(&world, pop, terminal);
    assert_eq!(result, AccessResult::DeniedDrift);
}

#[test]
fn test_access_slow_medium_drift() {
    let mut world = World::new();
    let pop = world
        .spawn((
            Pop,
            BiometricProfile {
                drift: 0.6,
                ..Default::default()
            }, // 60% Drift
        ))
        .id();

    let terminal = world
        .spawn(SecurityTerminal {
            required_clearance: 1,
            strictness: 1.0,
        })
        .id();

    let result = check_access(&world, pop, terminal);
    // Note: Delay duration is float, using assert matches pattern in AccessResult
    if let AccessResult::Delayed(duration) = result {
        assert!(duration > 0.0);
    } else {
        panic!("Expected Delayed access, got {:?}", result);
    }
}

#[test]
fn test_recalibration_resets_drift() {
    let mut world = World::new();
    let pop = world
        .spawn((
            Pop,
            BiometricProfile {
                drift: 0.9,
                last_update_tick: 0,
                recorded_scars: 0,
            },
        ))
        .id();

    world.insert_resource(SimulationTime {
        tick: 2000,
        speed: crate::shared::time::SimSpeed::Normal,
    });

    // Perform recalibration action
    recalibrate_profile(&mut world, pop);

    let profile = world.get::<BiometricProfile>(pop).unwrap();
    assert_eq!(profile.drift, 0.0);
    assert_eq!(profile.last_update_tick, 2000);
}
