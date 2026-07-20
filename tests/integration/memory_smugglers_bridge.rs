use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::memetics::memory_smugglers::{
    memory_smugglers_chronicle_bridge, MemeticDisassociation,
};
use scale::layer1::memetics::ReportedMemeticDisassociation;

#[test]
fn test_memory_smugglers_chronicle_bridge_triggers_event() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(memory_smugglers_chronicle_bridge);

    world.init_resource::<Events<AddChronicleEvent>>();

    let pop_entity = world.spawn(MemeticDisassociation { level: 100.0 }).id();

    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut found = false;

    for event in reader.read(events) {
        if event.text.contains("Memory Smugglers") || event.text.contains("memetic disassociation")
        {
            found = true;
            assert_eq!(event.importance, EventImportance::Major);
        }
    }

    assert!(
        found,
        "Expected AddChronicleEvent for memetic disassociation >= 100.0"
    );
    assert!(
        world
            .get::<ReportedMemeticDisassociation>(pop_entity)
            .is_some(),
        "Pop should receive ReportedMemeticDisassociation component to prevent duplicate events"
    );

    // Second run should not trigger another event
    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    assert_eq!(
        reader.read(events).count(),
        1,
        "Should not emit duplicate events"
    );
}
