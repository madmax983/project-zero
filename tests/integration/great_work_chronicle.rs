use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::construction::GreatWorkCompletedEvent;
use scale::layer1::integration::great_work_chronicle_bridge;

#[test]
fn test_great_work_chronicle_integration() {
    let mut world = World::new();

    // Setup events
    world.init_resource::<Events<GreatWorkCompletedEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    // Create schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(great_work_chronicle_bridge);

    // Act - send a GreatWorkCompletedEvent
    world.send_event(GreatWorkCompletedEvent {
        entity: Entity::from_raw(1),
        name: "Orbital Tether".to_string(),
    });

    schedule.run(&mut world);

    // Assert - should produce an AddChronicleEvent
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(
        events[0].importance,
        EventImportance::Legendary,
        "Great Work completion should be Legendary"
    );
    assert!(
        events[0].text.contains("Great Work"),
        "Text should mention the Great Work"
    );
}
