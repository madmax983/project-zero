use bevy::prelude::*;
use scale::cross_layer::generation_ship_mutiny::MutinyEvent;
use scale::layer1::core::chronicle::AddChronicleEvent;

#[test]
fn test_generation_ship_mutiny_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<MutinyEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, scale::cross_layer::generation_ship_mutiny::mutiny_chronicle_bridge);

    let ship_entity = app.world_mut().spawn_empty().id();
    app.world_mut().resource_mut::<Events<MutinyEvent>>().send(MutinyEvent { ship: ship_entity });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let chronicle_events: Vec<_> = reader.read(events).collect();

    assert_eq!(chronicle_events.len(), 1, "MutinyEvent should trigger AddChronicleEvent");
    assert!(chronicle_events[0].text.contains("Generation Ship Mutiny"), "Chronicle text should mention mutiny");
}
