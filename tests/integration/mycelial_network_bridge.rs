use bevy_ecs::prelude::*;
use scale::layer1::biology::genetics::crop_modification::{Crop, CropMutationEvent, MutationType};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::{crop_mutation_mycelial_bridge, mycelial_chronicle_bridge};
use scale::layer1::core::map::GridPosition;
use scale::layer1::logistics::mycelial::ContaminationEvent;

fn setup_app() -> World {
    let mut world = World::new();
    world.init_resource::<Events<CropMutationEvent>>();
    world.init_resource::<Events<ContaminationEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();
    world
}

#[test]
fn test_crop_mutation_triggers_contamination() {
    let mut world = setup_app();

    let crop = world
        .spawn((
            Crop {
                base_yield: 10,
                current_yield: 10,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    world
        .resource_mut::<Events<CropMutationEvent>>()
        .send(CropMutationEvent {
            crop_entity: crop,
            mutation_type: MutationType::ToxicSpores,
        });

    let mut schedule = Schedule::default();
    schedule.add_systems(crop_mutation_mycelial_bridge);
    schedule.run(&mut world);

    let contamination_events = world.resource::<Events<ContaminationEvent>>();
    assert_eq!(contamination_events.len(), 1);

    let mut event_cursor = contamination_events.get_cursor();
    let ev = event_cursor.read(contamination_events).next().unwrap();
    assert_eq!(ev.source.x, 5);
    assert_eq!(ev.source.y, 5);
}

#[test]
fn test_contamination_triggers_chronicle() {
    let mut world = setup_app();

    world
        .resource_mut::<Events<ContaminationEvent>>()
        .send(ContaminationEvent {
            source: GridPosition { x: 5, y: 5 },
        });

    let mut schedule = Schedule::default();
    schedule.add_systems(mycelial_chronicle_bridge);
    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    assert_eq!(chronicle_events.len(), 1);
}

#[test]
fn test_immune_response_triggers_chronicle() {
    let mut world = setup_app();
    world.spawn(scale::layer2::mycelial_network::SpaceFauna { strength: 100 });
    let mut schedule = Schedule::default();
    schedule
        .add_systems(scale::layer2::integration::mycelial_network_immune_response_chronicle_bridge);
    schedule.run(&mut world);
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    assert_eq!(chronicle_events.len(), 1);
}
