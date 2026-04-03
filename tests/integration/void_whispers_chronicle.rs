use bevy_ecs::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer2::exploration::void_whispers::{
    spread_whispers_to_colony, ColonyPop, FleetReturnedEvent, VoidWhispers,
};
use scale::layer2::integration::void_whispers_chronicle_bridge;

#[test]
fn test_void_whispers_returns_chronicle_event() {
    let mut app = bevy_app::App::new();
    app.add_event::<FleetReturnedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        bevy_app::Update,
        (spread_whispers_to_colony, void_whispers_chronicle_bridge).chain(),
    );

    let colony = app.world_mut().spawn_empty().id();

    // Need to spawn at least one pop to trigger the infection logic (if it requires one)
    app.world_mut().spawn((ColonyPop { colony },));

    let fleet = app
        .world_mut()
        .spawn((VoidWhispers { intensity: 50.0 },))
        .id();

    app.world_mut().send_event(FleetReturnedEvent { fleet, colony });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted = reader.read(events).collect::<Vec<_>>();

    assert_eq!(emitted.len(), 1, "Should emit exactly one chronicle event");
    assert!(
        emitted[0].text.contains("Void Whispers"),
        "Text should mention Void Whispers"
    );
}
