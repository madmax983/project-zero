use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::diplomacy::succession::{SuccessionCrisisEvent, SuccessionEvent};
use scale::layer3::integration::{
    dynastic_crisis_chronicle_bridge, dynastic_succession_chronicle_bridge,
};

#[test]
fn test_dynastic_succession_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<SuccessionEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, dynastic_succession_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<SuccessionEvent>>()
        .send(SuccessionEvent {
            faction_name: "Empire".to_string(),
            old_leader_name: "King A".to_string(),
            new_leader_name: "King B".to_string(),
        });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("King B"));
    assert_eq!(emitted[0].importance, EventImportance::Major);
}

#[test]
fn test_dynastic_crisis_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<SuccessionCrisisEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, dynastic_crisis_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<SuccessionCrisisEvent>>()
        .send(SuccessionCrisisEvent {
            faction_name: "Empire".to_string(),
            old_leader_name: "King A".to_string(),
        });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("crisis"));
    assert_eq!(emitted[0].importance, EventImportance::Legendary);
}
