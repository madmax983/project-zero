use bevy::prelude::*;
use scale::layer1::contagion::{contagion_system, ContagionType, EmotionalContagion};
use scale::layer1::map::GridPosition;
use scale::layer1::morale::{MoodModifier, Morale};

#[test]
fn test_contagion_spreads_to_nearby_pops() {
    let mut app = App::new();
    app.add_systems(Update, contagion_system);

    // Arrange: Two pops, one panicking, one neutral, within radius
    let _source = app
        .world_mut()
        .spawn((
            GridPosition { x: 0, y: 0 },
            Morale {
                value: 0.1,
                modifiers: vec![],
            }, // Very low morale
            EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5.0,
                strength: -0.15, // Modified from -15.0 to reflect Morale being [0.0, 1.0] internally
            },
        ))
        .id();

    let target = app
        .world_mut()
        .spawn((
            GridPosition { x: 3, y: 0 },
            Morale {
                value: 0.5,
                modifiers: vec![],
            },
        ))
        .id();

    // Act
    app.update();

    // Assert: Target should receive a negative mood modifier from contagion
    let target_morale = app.world().get::<Morale>(target).unwrap();
    assert!(
        target_morale
            .modifiers
            .iter()
            .any(|m| m.label == "Contagion: Panic"),
        "Target did not receive Panic contagion modifier"
    );
}

#[test]
fn test_contagion_ignores_distant_pops() {
    let mut app = App::new();
    app.add_systems(Update, contagion_system);

    // Arrange: Target is outside the 5.0 radius
    let _source = app
        .world_mut()
        .spawn((
            GridPosition { x: 0, y: 0 },
            EmotionalContagion {
                contagion_type: ContagionType::Joy,
                radius: 5.0,
                strength: 0.1, // Modified from 10.0 to reflect Morale being [0.0, 1.0]
            },
        ))
        .id();

    let target = app
        .world_mut()
        .spawn((
            GridPosition { x: 10, y: 0 },
            Morale {
                value: 0.5,
                modifiers: vec![],
            },
        ))
        .id();

    // Act
    app.update();

    // Assert: Target should NOT receive a mood modifier
    let target_morale = app.world().get::<Morale>(target).unwrap();
    assert!(
        !target_morale
            .modifiers
            .iter()
            .any(|m| m.label == "Contagion: Joy"),
        "Target incorrectly received contagion outside radius"
    );
}

#[test]
fn test_contagion_stacking_limits() {
    // Tests that a pop doesn't get infinitely depressed by standing next to a panicking pop for a long time.
    // Ensure the modifier resets or caps.
    let mut app = App::new();
    app.add_systems(Update, contagion_system);

    // Arrange: Two pops, one panicking, one neutral, within radius
    let _source = app
        .world_mut()
        .spawn((
            GridPosition { x: 0, y: 0 },
            Morale {
                value: 0.1,
                modifiers: vec![],
            },
            EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5.0,
                strength: -0.15,
            },
        ))
        .id();

    let target = app
        .world_mut()
        .spawn((
            GridPosition { x: 3, y: 0 },
            Morale {
                value: 0.5,
                modifiers: vec![],
            },
        ))
        .id();

    // Act - Run multiple times
    app.update();
    app.update();
    app.update();

    // Assert
    let target_morale = app.world().get::<Morale>(target).unwrap();
    let modifier_count = target_morale
        .modifiers
        .iter()
        .filter(|m| m.label == "Contagion: Panic")
        .count();
    assert_eq!(modifier_count, 1, "Modifiers should not stack infinitely");
}
