use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::economy::resources::{ColonyResources, ResourceType};
use scale::layer3::integration::quantum_famine_export_dump_bridge;
use scale::layer3::market::quantum_famine::ExportDumpEvent;

#[test]
fn test_quantum_famine_export_dump_bridge() {
    // Arrange
    let mut app = App::new();
    app.add_event::<ExportDumpEvent>();
    app.add_event::<AddChronicleEvent>();

    // Setup resources
    let mut resources = ColonyResources::default();
    resources.credits = 100.0;
    app.insert_resource(resources);

    app.add_systems(Update, quantum_famine_export_dump_bridge);

    let dummy_entity = app.world_mut().spawn_empty().id();

    // Act
    app.world_mut().send_event(ExportDumpEvent {
        stockpile_entity: dummy_entity,
        commodity: ResourceType::Food,
        amount_dumped: 450.0,
        credits_earned: 5000.0,
    });

    app.update();

    // Assert credits added
    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.credits, 5100.0, "Credits should have increased by 5000");

    // Assert Chronicle event generated
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Market panic caused speculative fleets"));
    assert!(events[0].text.contains("5000"));
}
