use bevy_ecs::prelude::*;
use scale::layer1::ancestral_graves::SacrilegeEvent;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::sacrilege_unrest_bridge;
use scale::layer1::map::GridPosition;
use scale::layer1::unrest::Unrest;

#[test]
fn test_sacrilege_unrest_bridge() {
    let mut world = World::new();

    // Setup events and resources
    world.init_resource::<Events<SacrilegeEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();
    world.insert_resource(Unrest::default());

    // Setup schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(sacrilege_unrest_bridge);

    // Send event
    world.send_event(SacrilegeEvent {
        pos: GridPosition { x: 0, y: 0 },
    });

    schedule.run(&mut world);

    // Assert Unrest increased
    let unrest = world.resource::<Unrest>();
    assert!(
        unrest.level > 0.0,
        "Global unrest should increase due to sacrilege"
    );

    // Assert Chronicle event emitted
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(
        events[0].importance,
        EventImportance::Major,
        "Sacrilege should be a Major event"
    );
    assert!(
        events[0].text.contains("grave"),
        "Text should mention the grave or sacrilege reason"
    );
}
