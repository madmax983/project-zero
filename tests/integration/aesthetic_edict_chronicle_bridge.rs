use bevy_ecs::prelude::*;
use scale::layer1::administration::edicts::{ColonyPolicies, Policy};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::aesthetic_edict_chronicle_bridge;

#[test]
fn aesthetic_edict_chronicle_bridge_emits_event() {
    let mut world = World::new();
    world.insert_resource(Events::<AddChronicleEvent>::default());
    world.insert_resource(ColonyPolicies::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(aesthetic_edict_chronicle_bridge);

    // Initial run, should be false -> false, no events
    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();
    assert_eq!(emitted.len(), 0);

    // Activate edict
    world.resource_mut::<ColonyPolicies>().active_policies.insert(Policy::Aesthetic);

    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();
    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("Aesthetic Edict"));

    // Deactivate edict
    world.resource_mut::<ColonyPolicies>().active_policies.remove(&Policy::Aesthetic);

    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();
    assert_eq!(emitted.len(), 1); // 1 new event
    assert_eq!(emitted[0].importance, EventImportance::Standard);
    assert!(emitted[0].text.contains("lifted"));
}
