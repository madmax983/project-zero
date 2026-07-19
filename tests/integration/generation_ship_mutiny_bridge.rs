use bevy::prelude::*;
use scale::cross_layer::generation_ship_mutiny::MutinyEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::generation_ship_mutiny_chronicle_bridge;

#[test]
fn test_generation_ship_mutiny_triggers_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<MutinyEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, generation_ship_mutiny_chronicle_bridge);

    let ship_entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(MutinyEvent { ship: ship_entity });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "MutinyEvent should trigger AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("generation ship"));
}
