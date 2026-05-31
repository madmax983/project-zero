use bevy_ecs::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::pop::Pop;
use scale::layer1::skills::Skills;
use scale::layer3::diplomacy::endless_draft::{
    DraftComplianceEvent, DraftOrderEvent, DraftRefusalEvent,
};
use scale::layer3::integration::endless_draft_bridge_system;

#[test]
fn test_draft_compliance() {
    scale::setup::init_task_pools();
    let mut world = World::new();

    world.init_resource::<Events<DraftOrderEvent>>();
    world.init_resource::<Events<DraftComplianceEvent>>();
    world.init_resource::<Events<DraftRefusalEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let sponsor = world.spawn_empty().id();

    // Spawn enough pops meeting the physical requirements
    let mut skills1 = Skills::default();
    skills1.xp.insert(scale::layer1::skills::SkillType::Mining, 2.0);
    skills1.xp.insert(scale::layer1::skills::SkillType::Forestry, 3.0);
    let pop1 = world
        .spawn((
            Pop,
            skills1,
        ))
        .id();

    let mut skills2 = Skills::default();
    skills2.xp.insert(scale::layer1::skills::SkillType::Mining, 3.0);
    skills2.xp.insert(scale::layer1::skills::SkillType::Forestry, 4.0);
    let pop2 = world
        .spawn((
            Pop,
            skills2,
        ))
        .id();

    world.send_event(DraftOrderEvent {
        sponsor,
        required_pops: 2,
        min_physical_stat: 4.0, // pop1 has 5.0, pop2 has 7.0
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(endless_draft_bridge_system);
    schedule.run(&mut world);

    // Verify pops were despawned
    assert!(
        world.get_entity(pop1).is_err(),
        "Pop1 should be despawned for the draft"
    );
    assert!(
        world.get_entity(pop2).is_err(),
        "Pop2 should be despawned for the draft"
    );

    // Verify compliance event was sent
    let compliance_events = world.resource::<Events<DraftComplianceEvent>>();
    let mut reader = compliance_events.get_cursor();
    let emitted: Vec<_> = reader.read(compliance_events).collect();
    assert_eq!(emitted.len(), 1, "Should emit one DraftComplianceEvent");
    assert_eq!(emitted[0].sponsor, sponsor);
    assert_eq!(emitted[0].pops_provided.len(), 2);

    // Verify chronicle event was sent
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let emitted: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert!(emitted[0]
        .text
        .contains("The colony has complied with the Draft Order"));
}

#[test]
fn test_draft_refusal() {
    scale::setup::init_task_pools();
    let mut world = World::new();

    world.init_resource::<Events<DraftOrderEvent>>();
    world.init_resource::<Events<DraftComplianceEvent>>();
    world.init_resource::<Events<DraftRefusalEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let sponsor = world.spawn_empty().id();

    // Spawn a pop that doesn't meet the physical requirements
    let mut skills1 = Skills::default();
    skills1.xp.insert(scale::layer1::skills::SkillType::Mining, 1.0);
    skills1.xp.insert(scale::layer1::skills::SkillType::Forestry, 1.0);
    let pop1 = world
        .spawn((
            Pop,
            skills1,
        ))
        .id();

    world.send_event(DraftOrderEvent {
        sponsor,
        required_pops: 2,
        min_physical_stat: 4.0, // pop1 only has 2.0
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(endless_draft_bridge_system);
    schedule.run(&mut world);

    // Verify pop was NOT despawned
    assert!(
        world.get_entity(pop1).is_ok(),
        "Pop1 should not be despawned"
    );

    // Verify refusal event was sent
    let refusal_events = world.resource::<Events<DraftRefusalEvent>>();
    let mut reader = refusal_events.get_cursor();
    let emitted: Vec<_> = reader.read(refusal_events).collect();
    assert_eq!(emitted.len(), 1, "Should emit one DraftRefusalEvent");
    assert_eq!(emitted[0].sponsor, sponsor);

    // Verify chronicle event was sent
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let emitted: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert!(emitted[0]
        .text
        .contains("The colony failed to meet the Draft Order quota"));
}
