use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::diplomatic_ward_death_chronicle_bridge;
use scale::layer1::diplomacy::wards::WarDeclaredEvent;
use scale::layer1::social::factions::FactionId;

#[test]
fn test_ward_death_chronicle_integration() {
    let mut app = App::new();

    app.add_event::<WarDeclaredEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, diplomatic_ward_death_chronicle_bridge);

    // Send a WarDeclaredEvent
    app.world_mut().send_event(WarDeclaredEvent {
        target_faction: FactionId::FarmersGuild,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let events_list: Vec<_> = cursor.read(events).collect();

    assert_eq!(
        events_list.len(),
        1,
        "Should emit exactly one chronicle event"
    );
    assert_eq!(events_list[0].importance, EventImportance::Major);
    assert!(
        events_list[0].text.contains("A diplomatic ward has perished"),
        "Event text should mention the diplomatic ward perishing"
    );
}
