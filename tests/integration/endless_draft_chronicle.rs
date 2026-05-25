use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::diplomacy::endless_draft::{DraftOrderEvent, DraftRefusalEvent};
use scale::layer3::integration::{endless_draft_chronicle_bridge, endless_draft_refusal_chronicle_bridge};

#[test]
fn test_endless_draft_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<DraftOrderEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, endless_draft_chronicle_bridge);

    let sponsor = Entity::from_raw(1);
    app.world_mut().send_event(DraftOrderEvent {
        sponsor,
        required_pops: 5,
        min_physical_stat: 10.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("A Total War empire has issued a Draft Order"));
    assert_eq!(emitted[0].importance, EventImportance::Major);
}

#[test]
fn test_endless_draft_refusal_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<DraftRefusalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, endless_draft_refusal_chronicle_bridge);

    let sponsor = Entity::from_raw(1);
    app.world_mut().send_event(DraftRefusalEvent { sponsor });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("The colony refused the Draft Order"));
    assert_eq!(emitted[0].importance, EventImportance::Major);
}
