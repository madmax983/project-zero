use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::cascade::{DefenseWeakenedEvent, LogisticsStrainedEvent};
use scale::layer2::integration::{
    defense_weakened_chronicle_bridge, logistics_strained_chronicle_bridge,
};

#[test]
fn test_logistics_strained_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<LogisticsStrainedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, logistics_strained_chronicle_bridge);

    let entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(LogisticsStrainedEvent {
        entity,
        capacity: 50,
        utilized: 100,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0]
        .text
        .contains("System logistics critically strained. Utilized 100/50 capacity."));
    assert_eq!(events[0].importance, EventImportance::Standard);
}

#[test]
fn test_defense_weakened_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<DefenseWeakenedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, defense_weakened_chronicle_bridge);

    let entity = app.world_mut().spawn_empty().id();

    app.world_mut()
        .send_event(DefenseWeakenedEvent { entity, power: 450 });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0]
        .text
        .contains("Sector defenses weakened to 450 power due to logistics failures."));
    assert_eq!(events[0].importance, EventImportance::Major);
}
