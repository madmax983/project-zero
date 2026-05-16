use bevy::app::{App, Update};
use bevy::ecs::prelude::*;
use scale::layer1::core::integration::emotional_contagion_trigger_bridge;
use scale::layer1::entities::pop::Pop;
use scale::layer1::social::emotional_contagion::{ContagionType, EmotionalContagion};
use scale::layer1::social::morale::Morale;

#[test]
fn test_extremely_low_morale_adds_panic_contagion() {
    let mut app = App::new();

    app.add_systems(Update, emotional_contagion_trigger_bridge);

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 0.1, // Below 0.15 threshold
                ..Default::default()
            },
        ))
        .id();

    app.update();

    let contagion = app.world().get::<EmotionalContagion>(pop_entity);
    assert!(contagion.is_some(), "Pop should have EmotionalContagion");
    assert_eq!(contagion.unwrap().contagion_type, ContagionType::Panic);
}

#[test]
fn test_extremely_high_morale_adds_joy_contagion() {
    let mut app = App::new();

    app.add_systems(Update, emotional_contagion_trigger_bridge);

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 0.9, // Above 0.85 threshold
                ..Default::default()
            },
        ))
        .id();

    app.update();

    let contagion = app.world().get::<EmotionalContagion>(pop_entity);
    assert!(contagion.is_some(), "Pop should have EmotionalContagion");
    assert_eq!(contagion.unwrap().contagion_type, ContagionType::Joy);
}

#[test]
fn test_normal_morale_removes_contagion() {
    let mut app = App::new();

    app.add_systems(Update, emotional_contagion_trigger_bridge);

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 0.5, // Normal morale
                ..Default::default()
            },
            EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5.0,
                strength: -0.1,
            },
        ))
        .id();

    app.update();

    let contagion = app.world().get::<EmotionalContagion>(pop_entity);
    assert!(contagion.is_none(), "Pop should not have EmotionalContagion");
}
