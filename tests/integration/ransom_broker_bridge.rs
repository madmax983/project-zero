use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::pop_lost_to_pirates_chronicle_bridge;
use scale::layer1::social::ransom_broker::PopLostToPiratesEvent;

#[test]
fn test_pop_lost_to_pirates_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<PopLostToPiratesEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, pop_lost_to_pirates_chronicle_bridge);

    let pop_entity = app.world_mut().spawn_empty().id();

    app.world_mut()
        .send_event(PopLostToPiratesEvent { pop_entity });
    app.update();

    let events = app
        .world()
        .get_resource::<Events<AddChronicleEvent>>()
        .unwrap();
    let mut cursor = events.get_cursor();
    assert_eq!(cursor.read(events).count(), 1);
}
