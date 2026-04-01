use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::social::ghost_shift_strike::GhostShiftStartedEvent;
use scale::layer1::integration::ghost_shift_chronicle_bridge;
use scale::shared::colony::ColonyName;
use scale::shared::narrative::NarrativeGenerator;

#[test]
fn test_ghost_shift_chronicle_bridge() {
    let mut app = App::new();

    // Init resources and events needed for the bridge
    app.insert_resource(ColonyName {
        name: "TestColony".to_string(),
    });
    app.insert_resource(NarrativeGenerator::from_embedded());
    app.add_event::<GhostShiftStartedEvent>();
    app.add_event::<AddChronicleEvent>();

    // Add the system
    app.add_systems(Update, ghost_shift_chronicle_bridge);

    let pop_entity = app.world_mut().spawn_empty().id();

    // Fire the event
    app.world_mut().send_event(GhostShiftStartedEvent { entity: pop_entity });

    // Run a cycle
    app.update();

    // Verify Chronicle event was emitted
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected one chronicle event to be fired");
    assert_eq!(
        events[0].importance,
        EventImportance::Major,
        "Ghost shift should be a major event"
    );
    assert!(events[0].text.contains("ghost shift") || events[0].text.contains("strike") || events[0].text.contains("quiet"), "Event text should mention the ghost shift/strike");
}
