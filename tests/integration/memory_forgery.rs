use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::truth_outbreak_chronicle_bridge;
use scale::layer1::psychology::memory_forgery::TruthOutbreakEvent;

#[test]
fn test_truth_outbreak_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<TruthOutbreakEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, truth_outbreak_chronicle_bridge);

    app.world_mut().send_event(TruthOutbreakEvent);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0]
        .text
        .contains("The truth has been revealed. A fabricated reality collapses as pops discover the memory forgery."));
    assert_eq!(events[0].importance, EventImportance::Major);
}