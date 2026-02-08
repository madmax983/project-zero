use scale::layer1::chronicle::{AddChronicleEvent, Chronicle, EventImportance};
use scale::layer1::pop::Pop;
use scale::layer1::rumor::{Knowledge, RumorTopic};
use scale::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[test]
fn test_chronicle_event_creates_rumor() {
    let mut world = World::new();
    world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
    world.insert_resource(Chronicle::default());
    world.init_resource::<Events<AddChronicleEvent>>();

    // Create schedule with only the relevant systems
    let mut schedule = Schedule::default();
    schedule.add_systems((
        scale::layer1::chronicle::chronicle_event_handler_system,
        scale::layer1::integration::chronicle_rumor_bridge_system,
    ));

    // Spawn 5 pops
    let mut pops = Vec::new();
    for _ in 0..5 {
        pops.push(world.spawn((Pop, Knowledge::default())).id());
    }

    // Send a Major Event
    world.send_event(AddChronicleEvent {
        text: "The Great Flood".to_string(),
        importance: EventImportance::Major,
    });

    // Run systems
    schedule.run(&mut world);

    // Verify Chronicle updated
    let chronicle = world.resource::<Chronicle>();
    assert_eq!(chronicle.events.len(), 1);
    assert_eq!(chronicle.events[0].text, "The Great Flood");

    // Verify Rumors spread (at least one pop should know it)
    let mut known_count = 0;
    for &pop in &pops {
        let knowledge = world.get::<Knowledge>(pop).unwrap();
        for rumor in &knowledge.known_rumors {
            if let RumorTopic::EventNews(text) = &rumor.topic {
                if text == "The Great Flood" {
                    known_count += 1;
                }
            }
        }
    }

    assert!(known_count > 0, "At least one pop should have heard the rumor");
    // Since we pick 3 witnesses out of 5, it should be exactly 3.
    assert_eq!(known_count, 3, "Should pick 3 witnesses");
}

#[test]
fn test_chronicle_minor_event_no_rumor() {
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(Chronicle::default());
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        scale::layer1::chronicle::chronicle_event_handler_system,
        scale::layer1::integration::chronicle_rumor_bridge_system,
    ));

    let pop = world.spawn((Pop, Knowledge::default())).id();

    world.send_event(AddChronicleEvent {
        text: "A squirrel ate a nut".to_string(),
        importance: EventImportance::Minor,
    });

    schedule.run(&mut world);

    // Chronicle updated
    assert_eq!(world.resource::<Chronicle>().events.len(), 1);

    // No rumor
    let knowledge = world.get::<Knowledge>(pop).unwrap();
    assert!(knowledge.known_rumors.is_empty());
}
