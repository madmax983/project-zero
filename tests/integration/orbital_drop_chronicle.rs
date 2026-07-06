use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::logistics::orbital_drop::orbital_drop_chronicle_bridge;
use scale::layer1::items::ItemType;
use scale::layer1::logistics::orbital_drop::OrbitalDropEvent;
use scale::layer1::map::GridPosition;

#[test]
fn test_orbital_drop_emits_chronicle_event() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Initialize required event streams
    app.init_resource::<Events<OrbitalDropEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    // Add the integration bridge system
    app.add_systems(Update, orbital_drop_chronicle_bridge);

    // Act
    let drop_event = OrbitalDropEvent {
        target: GridPosition { x: 10, y: 10 },
        items: vec![ItemType::Scrap, ItemType::Potato],
        scatter_radius: 5,
    };
    app.world_mut().send_event(drop_event);

    // Run the system to process the event
    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Orbital Drop should emit exactly one Chronicle Event"
    );

    let event = events[0];
    assert!(
        event.text.contains("Orbital Drop"),
        "Event text should describe the orbital drop, found: {}",
        event.text
    );
    assert!(
        event.text.contains("10, 10"),
        "Event text should include the drop coordinates, found: {}",
        event.text
    );
    assert!(
        event.text.contains("2 items"),
        "Event text should include the number of items, found: {}",
        event.text
    );
    assert_eq!(
        event.importance,
        EventImportance::Major,
        "Orbital drops should be considered Major events"
    );
}
