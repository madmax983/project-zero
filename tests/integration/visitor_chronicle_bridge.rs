use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::entities::the_visitor::{TheVisitor, TheVisitorState};
use scale::layer1::core::integration::visitor_chronicle_bridge;

#[test]
fn test_visitor_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<AddChronicleEvent>>();

    // Create a visitor component that just arrived
    let visitor = TheVisitor {
        state: TheVisitorState::Wander,
        ..Default::default()
    };

    // Setup system
    let mut schedule = Schedule::default();
    schedule.add_systems(visitor_chronicle_bridge);

    world.spawn(visitor);
    schedule.run(&mut world);

    // Should emit an event
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent for arrival");
    assert_eq!(emitted[0].text, "The Visitor has arrived.");
    assert_eq!(emitted[0].importance, EventImportance::Major);
}
