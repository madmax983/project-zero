use bevy::prelude::*;
use scale::layer1::core::integration::orphaned_swarm_chronicle_bridge;
use scale::layer2::events_new::orphaned_swarm::SwarmHostileEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};

#[test]
fn test_orphaned_swarm_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<SwarmHostileEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, orphaned_swarm_chronicle_bridge);

    app.world_mut().send_event(SwarmHostileEvent);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let events_read: Vec<_> = reader.read(events).collect();

    assert_eq!(events_read.len(), 1);
    assert_eq!(events_read[0].importance, EventImportance::Major);
    assert!(events_read[0].text.contains("Orphaned Swarm"));
}
