use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::architecture::potemkin::PotemkinDestroyedEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::potemkin_chronicle_bridge;

#[test]
fn test_potemkin_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<PotemkinDestroyedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, potemkin_chronicle_bridge);

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(PotemkinDestroyedEvent {
        entity,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut count = 0;
    for event in reader.read(events) {
        assert_eq!(event.importance, EventImportance::Minor);
        count += 1;
    }
    assert_eq!(count, 1);
}
