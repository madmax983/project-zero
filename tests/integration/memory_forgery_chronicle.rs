use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::psychology::memory_forgery::TruthOutbreakEvent;
use scale::layer1::core::integration::truth_outbreak_chronicle_bridge;

#[test]
fn test_truth_outbreak_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<TruthOutbreakEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, truth_outbreak_chronicle_bridge);

    app.world_mut().send_event(TruthOutbreakEvent);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("The fabricated truth has leaked"));
    assert_eq!(emitted[0].importance, EventImportance::Legendary);
}
