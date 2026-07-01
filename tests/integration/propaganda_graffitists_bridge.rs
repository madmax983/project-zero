use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::social::propaganda_graffitists::RebelliousGraffiti;

#[test]
fn test_propaganda_graffiti_chronicle_integration() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        scale::layer1::core::integration::propaganda_graffiti_chronicle_bridge,
    );

    app.world_mut().spawn(RebelliousGraffiti { intensity: 1.0 });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.len(), 1);

    let mut cursor = events.get_cursor();
    let ev = cursor.read(events).next().unwrap();
    assert_eq!(ev.importance, EventImportance::Standard);
    assert!(ev.text.contains("Subversive graffiti"));
}
