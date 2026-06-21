use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::{living_archive_chronicle_bridge, ChronicleCorruptedLogged};
use scale::layer1::tech::living_archive::Blueprint;

#[test]
fn test_living_archive_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, living_archive_chronicle_bridge);

    app.update(); // clear startup

    // Spawn a Blueprint that is corrupted
    let blueprint_entity = app
        .world_mut()
        .spawn(Blueprint {
            base_cost: 200.0,
            is_corrupted: true,
        })
        .id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit exactly one AddChronicleEvent when a Blueprint becomes corrupted"
    );
    assert!(
        emitted[0]
            .text
            .contains("A living Flesh-Server has suffered acute stress"),
        "Chronicle event text should mention the corrupted flesh server"
    );

    // Verify the marker component was added
    assert!(
        app.world()
            .get::<ChronicleCorruptedLogged>(blueprint_entity)
            .is_some(),
        "Blueprint should have the ChronicleCorruptedLogged marker"
    );

    // Another update should not trigger a second event
    app.update();
    let events2 = app.world().resource::<Events<AddChronicleEvent>>();
    let emitted2: Vec<&AddChronicleEvent> = cursor.read(events2).collect();
    assert_eq!(emitted2.len(), 0, "Should not emit again");
}
