use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::hazards::AmputationEvent;
use scale::layer1::pop::{Pop, PopName};
use scale::layer1::social::FavorChange;
use scale::layer1::integration::{amputation_handler_system, favor_chronicle_bridge};
use scale::shared::colony::ColonyName;
use scale::shared::log::MessageLog;
use scale::shared::narrative::NarrativeGenerator;
use scale::shared::time::SimulationTime;
use scale::layer1::memory::Memories;

#[test]
fn test_amputation_chronicle_integration() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Resources needed by amputation_handler_system
    world.init_resource::<Events<AmputationEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<MessageLog>();
    world.init_resource::<SimulationTime>();
    // NarrativeGenerator and ColonyName needed if we use them for text gen (maybe not yet)
    world.init_resource::<NarrativeGenerator>();
    world.init_resource::<ColonyName>();

    // System under test
    schedule.add_systems(amputation_handler_system);

    // Setup: A pop
    let pop = world.spawn((
        Pop,
        PopName("Stumpy".to_string()),
        Memories::default()
    )).id();

    // Act: Send event
    world.send_event(AmputationEvent { entity: pop });

    // Run system
    schedule.run(&mut world);

    // Assert: Check for chronicle event
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    // Currently fail because system is not modified yet
    assert_eq!(emitted.len(), 1, "Should emit 1 chronicle event");
    if !emitted.is_empty() {
        assert!(emitted[0].text.contains("Stumpy"), "Text should mention victim name");
        assert!(emitted[0].text.contains("limb"), "Text should mention limb loss");
        assert_eq!(emitted[0].importance, EventImportance::Major);
    }
}

#[test]
fn test_favor_chronicle_integration() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    world.init_resource::<Events<FavorChange>>();
    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<SimulationTime>();

    // System under test
    schedule.add_systems(favor_chronicle_bridge);

    // Setup: Two pops
    let pop_a = world.spawn((Pop, PopName("Alice".to_string()))).id();
    let pop_b = world.spawn((Pop, PopName("Bob".to_string()))).id();

    // Act: Send Major favor event
    world.send_event(FavorChange {
        debtor: pop_a,
        creditor: pop_b,
        amount: 50.0, // Major favor
        reason: "Saved from Fire".to_string(),
    });

    // Run system
    schedule.run(&mut world);

    // Assert
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    // Currently fail because system is a stub
    assert_eq!(emitted.len(), 1, "Should emit 1 chronicle event for major favor");
    if !emitted.is_empty() {
        assert!(emitted[0].text.contains("Alice"), "Should mention debtor");
        assert!(emitted[0].text.contains("Bob"), "Should mention creditor");
        assert_eq!(emitted[0].importance, EventImportance::Standard); // or Major
    }
}
