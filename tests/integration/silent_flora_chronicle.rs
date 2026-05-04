use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::silent_flora_chronicle_bridge;
use scale::layer1::flora::{Flora, FloraType};

#[test]
fn test_silent_flora_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, silent_flora_chronicle_bridge);

    app.world_mut().spawn(Flora {
        flora_type: FloraType::SilentFlora,
        ..Default::default()
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let iter = reader.read(events);
    assert!(iter.len() > 0, "Should have emitted an AddChronicleEvent");

    let event = iter.last().unwrap();
    assert_eq!(event.importance, EventImportance::Major);
}
