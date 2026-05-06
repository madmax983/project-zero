use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, Chronicle, EventImportance};
use scale::layer1::social::ghost_shift_strike::GhostShiftStartedEvent;
use scale::layer1::core::integration::ghost_shift_chronicle_bridge;

#[test]
fn ghost_shift_triggers_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<GhostShiftStartedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(Chronicle::default());

    app.add_systems(Update, ghost_shift_chronicle_bridge);

    let dummy = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<Events<GhostShiftStartedEvent>>()
        .send(GhostShiftStartedEvent { entity: dummy });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Ghost-Shift"));
}
