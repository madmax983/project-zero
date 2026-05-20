use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::ship::logistics::StrandedEvent;
use scale::layer3::integration::stranded_fleet_chronicle_bridge;

#[test]
fn test_stranded_fleet_emits_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<StrandedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, stranded_fleet_chronicle_bridge);

    app.world_mut().send_event(StrandedEvent {
        ship: Entity::from_raw(1),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).cloned().collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("stranded") || emitted[0].text.contains("Distress"));
}
