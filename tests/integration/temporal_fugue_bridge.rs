use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::temporal_fugue_chronicle_bridge;
use scale::layer1::mind::temporal_fugue::TemporalFugue;

#[test]
fn test_temporal_fugue_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, temporal_fugue_chronicle_bridge);

    app.update(); // clear startup

    // Spawn a pop with TemporalFugue
    app.world_mut().spawn(TemporalFugue);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit exactly one AddChronicleEvent when a pop enters TemporalFugue"
    );
    assert!(
        emitted[0]
            .text
            .contains("A highly skilled worker has entered a Temporal Fugue"),
        "Chronicle event text should mention the temporal fugue"
    );
}
