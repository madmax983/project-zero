use bevy::prelude::*;
use scale::layer1::administration::feral_administration::UnprocessedForms;
use scale::layer1::core::chronicle::AddChronicleEvent;

fn verify_event(mut events: EventReader<AddChronicleEvent>) {
    let emitted: Vec<_> = events.read().collect();
    assert_eq!(emitted.len(), 1);
    assert!(emitted[0]
        .text
        .contains("A mountain of Unprocessed Forms has collapsed"));
}

#[test]
fn test_feral_admin_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        (
            scale::layer1::administration::feral_administration::feral_admin_chronicle_bridge,
            verify_event,
        )
            .chain(),
    );

    // Act: Spawn an UnprocessedForm that crosses the impassable threshold
    app.world_mut()
        .spawn((UnprocessedForms { stack_size: 15 },));

    app.update();
}
