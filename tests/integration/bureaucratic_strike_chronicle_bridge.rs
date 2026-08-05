use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::bureaucratic_strike_chronicle_bridge;
use scale::layer1::social::bureaucratic_strike::RedTapeEvent;

#[test]
fn test_bureaucratic_strike_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(RedTapeEvent {
        active: false,
        severity: 1,
    });
    app.add_systems(Update, bureaucratic_strike_chronicle_bridge);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let reader = events.get_reader();
    assert!(reader.is_empty(events));

    app.world_mut().resource_mut::<RedTapeEvent>().active = true;
    app.world_mut().resource_mut::<RedTapeEvent>().severity = 10;

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();
    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("bureaucratic strike"));
    assert_eq!(emitted[0].importance, EventImportance::Major);

    app.world_mut().resource_mut::<RedTapeEvent>().active = false;
    app.world_mut().resource_mut::<RedTapeEvent>().severity = 1;

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();
    assert_eq!(emitted.len(), 2);
    assert!(emitted[1].text.contains("ended"));
    assert_eq!(emitted[1].importance, EventImportance::Standard);
}
