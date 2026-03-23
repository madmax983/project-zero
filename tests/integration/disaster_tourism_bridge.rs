use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::resources::ColonyResources;
use scale::layer1::map::GridPosition;
use scale::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent;

#[test]
fn grief_tourist_arrival_grants_credits_and_chronicles() {
    let mut world = World::new();
    world.insert_resource(Events::<GriefTouristArrivalEvent>::default());
    world.insert_resource(Events::<AddChronicleEvent>::default());
    world.insert_resource(ColonyResources {
        credits: 100.0,
        ..Default::default()
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(scale::layer2::integration::process_grief_tourist_arrival_system);

    world.send_event(GriefTouristArrivalEvent {
        target_location: GridPosition { x: 5, y: 5 },
        offered_credits: 50000.0,
    });

    schedule.run(&mut world);

    // Verify credits
    let resources = world.resource::<ColonyResources>();
    assert_eq!(resources.credits, 50100.0);

    // Verify chronicle event
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_reader();
    assert_eq!(reader.len(events), 1);

    let chronicle = reader.read(events).next().unwrap();
    assert!(chronicle.text.contains("Grief Tourists"));
    assert_eq!(chronicle.importance, EventImportance::Major);
}
