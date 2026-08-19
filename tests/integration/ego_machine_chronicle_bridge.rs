use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::{ego_stat_chronicle_bridge, EgoThresholdReached};
use scale::layer1::entities::pop::PopName;
use scale::layer1::tech::ego_machine::EgoStat;
use bevy::prelude::*;

#[test]
fn test_ego_stat_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, ego_stat_chronicle_bridge);

    // Should trigger event
    let dr_smith = app.world_mut().spawn((
        EgoStat { value: 60.0 },
        PopName("Dr. Smith".to_string()),
    )).id();

    // Shouldn't trigger event
    let intern = app.world_mut().spawn((
        EgoStat { value: 10.0 },
        PopName("Intern".to_string()),
    )).id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(emitted[0].text, "Dr. Smith demands a luxury suite, refusing to haul scrap.");

    assert!(app.world().get::<EgoThresholdReached>(dr_smith).is_some(), "Dr. Smith should have the marker component");
    assert!(app.world().get::<EgoThresholdReached>(intern).is_none(), "Intern should not have the marker component");

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should not emit duplicate events on subsequent ticks");
}
