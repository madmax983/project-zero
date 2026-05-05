use bevy_ecs::prelude::*;
use scale::layer1::access_control::{check_access, AccessControl, AccessMode};
use scale::layer1::pop::Pop;
use scale::layer1::security::{BiometricProfile, SecurityTerminal};

#[test]
fn test_biometric_drift_denies_access() {
    let mut world = World::new();

    let pop = world
        .spawn((
            Pop,
            BiometricProfile {
                drift: 0.9,
                ..Default::default()
            },
        ))
        .id();

    let terminal = world
        .spawn((
            SecurityTerminal {
                required_clearance: 1,
                strictness: 1.0,
            },
            AccessControl {
                mode: AccessMode::Public,
                ..Default::default()
            },
        ))
        .id();

    assert!(
        !check_access(&world, terminal, pop),
        "High drift should deny access"
    );
}
