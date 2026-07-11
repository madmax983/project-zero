use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::psychology::dreaming_sickness::DreamingSickness;

fn verify_event(mut events: EventReader<AddChronicleEvent>) {
    let emitted: Vec<_> = events.read().collect();
    assert_eq!(emitted.len(), 1);
    assert!(emitted[0]
        .text
        .contains("A strange dreaming sickness has begun to spread"));
}

#[test]
fn test_dreaming_sickness_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        (
            scale::layer1::core::integration::dreaming_sickness_chronicle_bridge,
            verify_event,
        )
            .chain(),
    );

    app.world_mut().spawn((DreamingSickness { severity: 0.1 },));

    app.update();
}
