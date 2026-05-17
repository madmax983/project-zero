use bevy::app::App;

use scale::layer1::social::emotional_contagion::{
    trigger_emotional_contagion_system, ContagionType, EmotionalContagion,
};
use scale::layer1::social::morale::Morale;

#[test]
fn test_emotional_contagion_triggers_panic() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, trigger_emotional_contagion_system);

    let pop = app
        .world_mut()
        .spawn(Morale {
            value: 10.0,
            ..Default::default()
        })
        .id();

    app.update();

    let contagion = app
        .world()
        .get::<EmotionalContagion>(pop)
        .expect("Should add EmotionalContagion");
    assert_eq!(contagion.contagion_type, ContagionType::Panic);
}

#[test]
fn test_emotional_contagion_triggers_joy() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, trigger_emotional_contagion_system);

    let pop = app
        .world_mut()
        .spawn(Morale {
            value: 90.0,
            ..Default::default()
        })
        .id();

    app.update();

    let contagion = app
        .world()
        .get::<EmotionalContagion>(pop)
        .expect("Should add EmotionalContagion");
    assert_eq!(contagion.contagion_type, ContagionType::Joy);
}

#[test]
fn test_emotional_contagion_removed_when_recovered() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, trigger_emotional_contagion_system);

    let pop = app
        .world_mut()
        .spawn((
            Morale {
                value: 50.0,
                ..Default::default()
            },
            EmotionalContagion {
                contagion_type: ContagionType::Panic,
                radius: 5.0,
                strength: -15.0,
            },
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<EmotionalContagion>(pop).is_none(),
        "Should remove EmotionalContagion when morale is normal"
    );
}
