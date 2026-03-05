use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::unrest::{DenounceEvent, ScapegoatAction};
use scale::layer1::integration::scapegoat_chronicle_bridge;

#[test]
fn test_scapegoat_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DenounceEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, scapegoat_chronicle_bridge);

    let target = app.world_mut().spawn_empty().id();

    // Send a DenounceEvent
    app.world_mut().send_event(DenounceEvent {
        target,
        action: ScapegoatAction::Exile,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();

    let chronicle_events: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(chronicle_events.len(), 1, "Should have produced exactly one AddChronicleEvent");

    let ev = chronicle_events[0];
    assert_eq!(ev.importance, EventImportance::Major, "Event importance should be Major");
    assert!(ev.text.contains("exiled"), "Text should mention exile. Got: {}", ev.text);
}
