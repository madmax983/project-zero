use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, Chronicle};
use scale::layer1::pop::{Pop, PopBorn, PopDied};
use scale::layer1::quantum_twins::QuantumTwin;
use scale::shared::time::SimulationTime;
use scale::shared::colony::ColonyName;
use scale::shared::narrative::NarrativeGenerator;
use scale::layer1::integration::{UnpairedClone, quantum_twin_clone_bridge, quantum_twin_severance_chronicle_bridge};

#[test]
fn test_clone_vat_creates_twins() {
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(UnpairedClone { entity: None });
    world.init_resource::<Events<PopBorn>>();

    // Create schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(quantum_twin_clone_bridge);

    let pop1 = world.spawn(Pop).id();
    let pop2 = world.spawn(Pop).id();
    let pop3 = world.spawn(Pop).id();

    // Fire event for first clone
    world.send_event(PopBorn {
        entity: pop1,
        name: "Clone A".to_string(),
        tick: 0,
        source: "Clone Vat".to_string(),
    });

    schedule.run(&mut world);

    // Should not be linked yet
    assert!(world.get::<QuantumTwin>(pop1).is_none());
    assert_eq!(world.resource::<UnpairedClone>().entity, Some(pop1));

    // Fire event for second clone
    world.send_event(PopBorn {
        entity: pop2,
        name: "Clone B".to_string(),
        tick: 1,
        source: "Clone Vat".to_string(),
    });

    schedule.run(&mut world);

    // Should be linked now
    let twin1 = world.get::<QuantumTwin>(pop1).expect("Pop1 should have QuantumTwin");
    assert_eq!(twin1.partner, pop2);
    let twin2 = world.get::<QuantumTwin>(pop2).expect("Pop2 should have QuantumTwin");
    assert_eq!(twin2.partner, pop1);
    assert_eq!(world.resource::<UnpairedClone>().entity, None);

    // Fire event for third clone but not from Vat
    world.send_event(PopBorn {
        entity: pop3,
        name: "Immigrant C".to_string(),
        tick: 2,
        source: "Ship".to_string(),
    });

    schedule.run(&mut world);

    // Should not affect unpaired
    assert_eq!(world.resource::<UnpairedClone>().entity, None);
    assert!(world.get::<QuantumTwin>(pop3).is_none());
}

#[test]
fn test_severance_chronicle() {
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(NarrativeGenerator::from_embedded());
    world.insert_resource(ColonyName {
        name: "Test Colony".to_string(),
    });

    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<Events<PopDied>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(quantum_twin_severance_chronicle_bridge);

    let pop1 = world.spawn(Pop).id();
    let pop2 = world.spawn(Pop).id();

    world.entity_mut(pop1).insert(QuantumTwin {
        partner: pop2,
        link_strength: 1.0,
    });
    world.entity_mut(pop2).insert(QuantumTwin {
        partner: pop1,
        link_strength: 1.0,
    });

    world.send_event(PopDied {
        entity: pop1,
        name: "Twin A".to_string(),
        tick: 10,
        reason: "Old Age".to_string(),
    });

    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert!(
        !emitted.is_empty(),
        "Death of a twin should trigger a severance chronicle event"
    );
    assert!(
        emitted[0].text.contains("Severance"),
        "Event text should contain Severance: {}",
        emitted[0].text
    );
    assert!(
        emitted[0].text.contains("Twin A"),
        "Event text should contain dead twin name: {}",
        emitted[0].text
    );
}
