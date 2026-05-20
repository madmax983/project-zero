use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::integration::stranded_chronicle_bridge;
use scale::layer2::ship::logistics::StrandedEvent;

#[test]
fn test_stranded_event_creates_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<StrandedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, stranded_chronicle_bridge);

    // Spawn a dummy entity to act as our stranded ship
    let ship_entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(StrandedEvent { ship: ship_entity });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let add_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(add_events.len(), 1, "Should create one AddChronicleEvent");
    assert!(
        add_events[0].text.contains("A ship has been stranded"),
        "Text should mention the stranded ship"
    );
    assert_eq!(
        add_events[0].importance,
        EventImportance::Major,
        "Importance should be Major"
    );
}
