use bevy_app::App;
use bevy_ecs::event::Events;
use scale::layer1::cassandra_protocol::{CassandraProtocolActive, Colony};
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::cassandra_protocol_chronicle_bridge;

#[test]
fn test_cassandra_protocol_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, cassandra_protocol_chronicle_bridge);

    let colony = app.world_mut().spawn(Colony).id();

    // Act
    app.world_mut().entity_mut(colony).insert(CassandraProtocolActive);
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit a chronicle event on Cassandra Protocol activation");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Cassandra Protocol"));
}
