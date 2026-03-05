use bevy_ecs::prelude::*;
use scale::layer1::edicts::{ColonyPolicies, Policy};
use scale::layer1::factions::{FactionId, FactionMember};
use scale::layer1::pop::{Job, Pop};
use scale::layer1::resources::ColonyResources;
use scale::layer1::social::pen_pals::{update_pen_pals_system, RemoteBond};
use scale::layer1::utility_types::AssignmentType;

#[test]
fn test_intel_gain_requires_comms_console_integration() {
    let mut world = World::new();
    world.insert_resource(ColonyResources::default());
    world.insert_resource(ColonyPolicies::default());

    let pop = world.spawn((Pop, FactionMember { faction_id: None })).id(); // No Job
    world.spawn(RemoteBond {
        local_pop: pop,
        foreign_faction: FactionId::FarmersGuild,
        affinity: 50.0,
        is_spy: false,
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(update_pen_pals_system);
    schedule.run(&mut world);

    let res = world.resource::<ColonyResources>();
    assert_eq!(
        res.knowledge, 0.0,
        "Should not gain knowledge if not working at comms"
    );
}

#[test]
fn test_ethics_shift_integration() {
    let mut world = World::new();
    world.insert_resource(ColonyResources::default());
    world.insert_resource(ColonyPolicies::default());
    let pop = world
        .spawn((
            Pop,
            Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::LibraryWorker,
            },
            FactionMember {
                faction_id: Some(FactionId::MinersGuild),
            },
        ))
        .id();

    world.spawn(RemoteBond {
        local_pop: pop,
        foreign_faction: FactionId::FarmersGuild,
        affinity: 100.0,
        is_spy: false,
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(update_pen_pals_system);
    schedule.run(&mut world);

    let member = world.get::<FactionMember>(pop).unwrap();
    assert_eq!(member.faction_id, Some(FactionId::FarmersGuild));
}

#[test]
fn test_firewall_comms_policy_integration() {
    let mut world = World::new();
    world.insert_resource(ColonyResources::default());
    let mut policies = ColonyPolicies::default();
    policies.active_policies.insert(Policy::FirewallComms);
    world.insert_resource(policies);

    let _pop = world
        .spawn((
            Pop,
            Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::LibraryWorker,
            },
            FactionMember { faction_id: None },
        ))
        .id();
    let pop = world
        .spawn((
            Pop,
            Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::LibraryWorker,
            },
            FactionMember { faction_id: None },
        ))
        .id();
    world.spawn(RemoteBond {
        local_pop: pop,
        foreign_faction: FactionId::FarmersGuild,
        affinity: 50.0,
        is_spy: false,
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(update_pen_pals_system);
    schedule.run(&mut world);

    let res = world.resource::<ColonyResources>();
    assert_eq!(
        res.knowledge, 0.0,
        "Knowledge gain should be blocked by Firewall"
    );
}

#[test]
fn test_espionage_loss_integration() {
    let mut world = World::new();
    let mut resources = ColonyResources::default();
    resources.knowledge = 100.0;
    world.insert_resource(resources);
    world.insert_resource(ColonyPolicies::default());

    let pop = world
        .spawn((
            Pop,
            Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::LibraryWorker,
            },
            FactionMember { faction_id: None },
        ))
        .id();
    world.spawn(RemoteBond {
        local_pop: pop,
        foreign_faction: FactionId::FarmersGuild,
        affinity: 50.0,
        is_spy: true, // Spy!
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(update_pen_pals_system);
    // Run several times to hit the chance
    for _ in 0..100 {
        schedule.run(&mut world);
    }

    let res = world.resource::<ColonyResources>();
    assert!(
        res.knowledge < 100.0,
        "Knowledge should be lost due to espionage"
    );
}
