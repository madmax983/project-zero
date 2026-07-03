use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::scrap_code_cult_formation_chronicle_bridge;
use scale::layer1::social::scrap_code_prophets::CultFormationEvent;

#[test]
fn test_scrap_code_cult_formation_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<CultFormationEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, scrap_code_cult_formation_chronicle_bridge);

    app.world_mut().send_event(CultFormationEvent {
        pop: Entity::from_raw(1),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(
        emitted[0].importance,
        EventImportance::Major,
        "Importance should be major"
    );
    assert!(
        emitted[0].text.contains("Cult of the Broken Machine"),
        "Chronicle text should mention the cult"
    );
}
