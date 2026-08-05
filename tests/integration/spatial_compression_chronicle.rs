use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::spatial_compression::PocketCollapseEvent;

#[test]
fn test_spatial_compression_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<PocketCollapseEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, scale::layer1::core::integration::spatial_compression_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<PocketCollapseEvent>>()
        .send(PocketCollapseEvent { pocket: Entity::PLACEHOLDER });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut found = false;
    for event in reader.read(events) {
        assert!(event.text.contains("Pocket Dimension collapsed"));
        assert!(matches!(event.importance, EventImportance::Major));
        found = true;
    }
    assert!(found, "Chronicle event should have been emitted for pocket collapse.");
}
