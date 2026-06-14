use bevy_ecs::prelude::*;
use scale::layer1::architecture::building::{BuildingType, MaterialType, spawn_building};
use scale::layer1::architecture::edible::{EdibleMaterial, Consumed};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::edible_architecture_chronicle_bridge;

#[test]
fn test_edible_architecture_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<AddChronicleEvent>>();

    // 1. Spawn a building with MaterialType::Wood
    let entity = spawn_building(&mut world, 0, 0, BuildingType::Housing, MaterialType::Wood);

    // 2. Verify it has EdibleMaterial
    assert!(
        world.get::<EdibleMaterial>(entity).is_some(),
        "Building made of Wood should have EdibleMaterial component"
    );

    // 3. Mark it as Consumed
    world.entity_mut(entity).insert(Consumed);

    // 4. Run the bridge system
    let mut schedule = Schedule::default();
    world.init_resource::<Events<scale::layer1::events::BuildingRemovedEvent>>();
    schedule.add_systems((edible_architecture_chronicle_bridge, scale::layer1::architecture::edible::consume_building_system).chain());
    schedule.run(&mut world);

    // 5. Verify the chronicle event was sent
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let iter: Vec<_> = reader.read(events).collect();
    assert_eq!(iter.len(), 1, "Should have emitted exactly 1 chronicle event");
    assert!(
        iter[0].text.contains("Housing architecture"),
        "Event text should mention the building type"
    );

    // 6. Verify food was produced
    let mut found_food = false;
    for item in world.query::<&scale::layer1::economy::items::Item>().iter(&world) {
        if item.item_type == scale::layer1::economy::items::ItemType::Potato {
            found_food = true;
            break;
        }
    }
    assert!(found_food, "Consuming the building should have yielded food items");
}
