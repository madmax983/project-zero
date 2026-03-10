use bevy_ecs::prelude::*;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::chronicle::Chronicle;
use scale::layer1::dreams::{dream_system, DreamJournal, DreamtThisSleep};
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::layer1::tech::hypno_learning::SleepingInHypnoPod;
use scale::layer1::utility_types::{ActionType, PopAction};
use scale::shared::log::MessageLog;
use scale::shared::narrative::NarrativeGenerator;
use scale::shared::time::SimulationTime;

#[test]
fn test_hypno_sleep_triggers_dreams_frequently() {
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(NarrativeGenerator::default());
    world.insert_resource(MessageLog::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(dream_system);

    let bed = world.spawn_empty().id();
    let pop = world
        .spawn((
            Pop,
            AssignedTo {
                entity: bed,
                assignment_type: AssignmentType::HousingResident,
            },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
            Needs::default(),
            SleepingInHypnoPod, // This makes the pop sleep in a HypnoPod
        ))
        .id();

    // Run system multiple times. With 5% chance, it should trigger quite fast compared to 1%.
    let mut dreamt = false;
    for _ in 0..100 {
        schedule.run(&mut world);
        if world.get::<DreamtThisSleep>(pop).is_some() {
            dreamt = true;
            break;
        }
    }

    assert!(
        dreamt,
        "Pop sleeping in HypnoPod should dream more frequently"
    );
    let journal = world.get::<DreamJournal>(pop);
    assert!(journal.is_some(), "DreamJournal should be created");
}

#[test]
fn test_hypno_nightmare_chance_is_higher() {
    // This is a statistical test, we will run it 100 times and expect at least one nightmare.
    // Given 50% nightmare chance for hypno pods (we will implement this), running it many times guarantees nightmares.
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(NarrativeGenerator::default());
    world.insert_resource(MessageLog::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(dream_system);

    let bed = world.spawn_empty().id();

    let mut nightmares = 0;

    for _ in 0..50 {
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: bed,
                    assignment_type: AssignmentType::HousingResident,
                },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                Needs::default(),
                SleepingInHypnoPod,
            ))
            .id();

        // Run until they dream
        for _ in 0..100 {
            schedule.run(&mut world);
            if world.get::<DreamtThisSleep>(pop).is_some() {
                if let Some(journal) = world.get::<DreamJournal>(pop) {
                    if let Some(dream) = &journal.last_dream {
                        if dream.is_nightmare {
                            nightmares += 1;
                        }
                    }
                }
                break;
            }
        }
    }

    assert!(nightmares > 0, "HypnoPods should cause nightmares");
}
