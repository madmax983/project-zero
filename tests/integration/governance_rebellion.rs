use bevy::MinimalPlugins;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::governance::RebellionEvent;
use scale::layer2::integration::rebellion_chronicle_bridge_system;

#[test]
fn test_rebellion_triggers_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<RebellionEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, rebellion_chronicle_bridge_system);

    let planet_entity = app.world_mut().spawn_empty().id();

    // Act: Send a RebellionEvent
    app.world_mut()
        .send_event(RebellionEvent { planet_entity });
    app.update();

    // Assert: We should receive a major chronicle event
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();

    let mut found = false;
    for event in reader.read(chronicle_events) {
        if event.importance == EventImportance::Major
            && event.text.contains("rebellion")
        {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "A RebellionEvent should trigger an AddChronicleEvent with EventImportance::Major"
    );
}
