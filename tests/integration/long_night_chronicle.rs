use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::long_night_chronicle_bridge;
use scale::layer1::nature::long_night::{LongNightEvent, StartLongNightEvent};
use bevy::prelude::*;

#[test]
fn test_long_night_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_event::<StartLongNightEvent>();
    app.init_resource::<LongNightEvent>();

    app.add_systems(Update, long_night_chronicle_bridge);

    // Trigger start event
    app.world_mut().send_event(StartLongNightEvent { duration_ticks: 100 });
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].text, "The Long Night has begun. The sun is blocked, and temperatures are plummeting.");

    // Set is_active to true to simulate the event being active
    app.world_mut().resource_mut::<LongNightEvent>().is_active = true;
    app.update();

    // Set is_active to false to simulate the event ending
    app.world_mut().resource_mut::<LongNightEvent>().is_active = false;
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let events: Vec<&AddChronicleEvent> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].text, "The Long Night has ended. The sun has finally returned.");
}
