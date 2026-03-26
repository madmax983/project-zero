use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::mass_driver_chronicle_bridge;
use scale::layer1::logistics::mass_driver::BombardmentEvent;

#[test]
fn test_bombardment_emits_chronicle_event() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Initialize required event streams
    app.init_resource::<Events<BombardmentEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    // Add the integration bridge system
    app.add_systems(Update, mass_driver_chronicle_bridge);

    // Act
    let target_entity = app.world_mut().spawn_empty().id();
    let bomb_event = BombardmentEvent {
        target: target_entity,
        kinetic_energy: 5000,
    };
    app.world_mut().send_event(bomb_event);

    // Run the system to process the event
    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Bombardment should emit exactly one Chronicle Event"
    );

    let event = events[0];
    assert!(
        event.text.contains("Kinetic Bombardment!"),
        "Event text should describe the bombardment, found: {}",
        event.text
    );
    assert!(
        event.text.contains("5000"),
        "Event text should include the kinetic energy, found: {}",
        event.text
    );
    assert_eq!(
        event.importance,
        EventImportance::Major,
        "Bombardment should be considered a Major event"
    );
}
