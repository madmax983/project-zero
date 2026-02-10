use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, Chronicle};
use scale::layer1::health::{Health, death_system};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::{Pop, PopDied, PopName};
use scale::shared::colony::ColonyName;
use scale::shared::log::MessageLog;
use scale::shared::narrative::NarrativeGenerator;
use scale::shared::time::SimulationTime;

#[test]
fn test_pop_death_adds_chronicle_entry() {
    let mut world = World::new();

    // Setup resources
    world.insert_resource(SimulationTime::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(NarrativeGenerator::from_embedded());
    world.insert_resource(ColonyName {
        name: "Test Colony".to_string(),
    });

    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<Events<PopDied>>();

    // Register the system that causes the event (death_system)
    let mut schedule = Schedule::default();
    schedule.add_systems((
        death_system,
        scale::layer1::integration::pop_death_chronicle_bridge.after(death_system),
    ));

    // Spawn a pop with low health
    let pop = world
        .spawn((
            Pop,
            PopName("TestSubject".to_string()),
            GridPosition::default(),
            Health {
                current: 0.0,
                max: 100.0,
            }, // Dead
        ))
        .id();

    // Run the schedule once
    schedule.run(&mut world);

    // Check that the pop is dead (despawned)
    assert!(world.get_entity(pop).is_err(), "Pop should be despawned");

    // Check for Chronicle Event
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert!(
        !emitted.is_empty(),
        "Death should trigger a chronicle event"
    );
    assert!(
        emitted[0].text.contains("TestSubject"),
        "Event text should contain pop name: {}",
        emitted[0].text
    );
}
