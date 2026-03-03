use bevy_ecs::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::tech::infinite_archive::{purge_tech, Archive};
use scale::layer1::tech::{Tech, TechState, TechStatus};

#[test]
fn test_purge_tech_creates_chronicle_event() {
    let mut world = World::new();

    // Setup requirements
    world.insert_resource(scale::shared::log::MessageLog::default());
    world.insert_resource(Archive::default());
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut tech_state = TechState::default();
    tech_state.total_capacity = 100.0;
    tech_state
        .techs
        .insert(Tech::VoidWhispers, TechStatus::Active);
    tech_state.update_corruption();
    world.insert_resource(tech_state);

    // Call function
    purge_tech(&mut world, "Void Whispers");

    // Check for event
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("Void Whispers"));
    assert!(emitted[0].text.contains("We have forgotten the secrets of"));
}
