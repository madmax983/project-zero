use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::clone_vat::{process_clone_vats_system, CloneVat};
use scale::layer1::housing::Housing;
use scale::layer1::map::GridPosition;
use scale::layer1::needs::{decay_needs_system, Needs};
use scale::layer1::notifications::NotificationQueue;
use scale::layer1::pop::{Pop, PopBorn};
use scale::layer1::resources::ColonyResources;
use scale::layer1::traits::{Trait, Traits};
use scale::shared::time::SimulationTime;

fn setup_world() -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(ColonyResources::default());
    world.init_resource::<Events<PopBorn>>();
    world.insert_resource(NotificationQueue::default());

    // Minimal resources for systems
    world.insert_resource(scale::layer1::terrain::TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
    });

    world
}

#[test]
fn test_clone_vat_emits_event_and_notification() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    world.insert_resource(SimulationTime {
        tick: 100,
        ..Default::default()
    });

    // Spawn Clone Vat ready to finish
    world.spawn((
        Building {
            building_type: BuildingType::CloneVat,
        },
        CloneVat {
            is_growing: true,
            ticks_remaining: 1,
            total_duration: 100,
            ..Default::default()
        },
        GridPosition { x: 5, y: 5 },
    ));

    // Run system
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            process_clone_vats_system,
            scale::layer1::integration::pop_born_notification_system,
        )
            .chain(),
    );

    schedule.run(&mut world);

    // Verify Event
    let events = world.resource::<Events<PopBorn>>();
    let mut reader = events.get_cursor();
    let events: Vec<_> = reader.read(events).collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, "Clone Vat");
    assert_eq!(events[0].tick, 100);

    // Verify Notification
    let queue = world.resource::<NotificationQueue>();
    assert!(!queue.active.is_empty());
    let notification = &queue.active[0];
    assert!(notification.text.contains("has been born"));
    assert!(notification.text.contains("Source: Clone Vat"));
}

#[test]
fn test_clone_auto_assigns_housing() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

    // Spawn Housing with capacity
    let housing_entity = world
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            Housing {
                capacity: 2,
                residents: vec![],
            },
            GridPosition { x: 0, y: 0 },
        ))
        .id();

    // Spawn Clone Vat ready to finish
    world.spawn((
        Building {
            building_type: BuildingType::CloneVat,
        },
        CloneVat {
            is_growing: true,
            ticks_remaining: 1,
            total_duration: 100,
            ..Default::default()
        },
        GridPosition { x: 5, y: 5 },
    ));

    world.run_system_once(process_clone_vats_system).unwrap();

    // Verify Pop assigned to Housing
    let pop_entity = world.query_filtered::<Entity, With<Pop>>().single(&world);
    let assigned = world.get::<AssignedTo>(pop_entity);

    assert!(assigned.is_some(), "Clone should be assigned to housing");
    let assigned = assigned.unwrap();
    assert_eq!(assigned.entity, housing_entity);
    assert_eq!(assigned.assignment_type, AssignmentType::HousingResident);

    // Verify Housing has resident
    let housing = world.get::<Housing>(housing_entity).unwrap();
    assert_eq!(housing.residents.len(), 1);
    assert_eq!(housing.residents[0], pop_entity);
}

#[test]
fn test_soulless_trait_reduces_leisure_decay() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

    // Spawn Soulless Pop
    let soulless_pop = world
        .spawn((Pop, Needs::default(), {
            let mut t = Traits::default();
            t.add(Trait::Soulless);
            t
        }))
        .id();

    // Spawn Normal Pop
    let normal_pop = world.spawn((Pop, Needs::default(), Traits::default())).id();

    // Run decay system
    world.run_system_once(decay_needs_system).unwrap();

    let soulless_needs = world.get::<Needs>(soulless_pop).unwrap();
    let normal_needs = world.get::<Needs>(normal_pop).unwrap();

    // 0.8 start.
    // Normal decay: 0.0015
    // Soulless decay: 0.0015 * 0.5 = 0.00075

    let normal_loss = 0.8 - normal_needs.leisure;
    let soulless_loss = 0.8 - soulless_needs.leisure;

    assert!(
        soulless_loss < normal_loss,
        "Soulless should lose less leisure"
    );
    assert!(
        (soulless_loss - (normal_loss * 0.5)).abs() < 0.0001,
        "Should be roughly half decay"
    );
}
