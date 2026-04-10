use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::law::justice::{CrimeCommittedEvent, CrimeRecord, CrimeType};
use crate::layer1::law::justice::ArrestEvent;
use crate::layer1::diplomacy::{DiplomaticImmunity, DiplomaticIncidentEvent};
use crate::layer1::law::embassy::{evaluate_diplomatic_crime_system, process_diplomatic_arrest_system};
use crate::layer1::social::factions::FactionId;

fn setup_world() -> World {
    let mut world = World::new();
    world.init_resource::<Events<CrimeCommittedEvent>>();
    world.init_resource::<Events<ArrestEvent>>();
    world.init_resource::<Events<DiplomaticIncidentEvent>>();
    world
}

#[test]
fn test_diplomat_commits_crime_ignored_by_sheriff() {
    let mut world = setup_world();

    // Spawn a diplomat pop
    let diplomat = world.spawn((
        Pop,
        DiplomaticImmunity { faction_id: FactionId::FarmersGuild },
        CrimeRecord::default()
    )).id();

    // Trigger a crime
    world.send_event(CrimeCommittedEvent {
        perpetrator: diplomat,
        crime_type: CrimeType::Assault, // Represents Major
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(evaluate_diplomatic_crime_system);
    schedule.run(&mut world);

    // A diplomat committing a crime should NOT get a Wanted token
    // that triggers Sheriff tasks automatically, to prevent AI sheriffs from starting wars.
    let record = world.get::<CrimeRecord>(diplomat).unwrap();
    assert!(!record.is_wanted());
}

#[test]
fn test_player_forced_arrest_triggers_diplomatic_incident() {
    let mut world = setup_world();

    let diplomat = world.spawn((
        Pop,
        DiplomaticImmunity { faction_id: FactionId::FarmersGuild },
    )).id();

    // The player manually orders an arrest despite immunity
    world.send_event(ArrestEvent {
        target: diplomat,
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(process_diplomatic_arrest_system);
    schedule.run(&mut world);

    // This must trigger a Layer 3 incident
    let incidents = world.resource::<Events<DiplomaticIncidentEvent>>();
    let mut reader = incidents.get_cursor();
    let iter: Vec<_> = reader.read(incidents).collect();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter[0].faction_id, FactionId::FarmersGuild);
}
