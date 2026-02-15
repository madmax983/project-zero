use bevy_ecs::prelude::*;
use scale::layer1::beauty::BeautyGrid;
use scale::layer1::map::GridPosition;
use scale::layer1::memory::{Memories, MemoryType};
use scale::layer1::pop::Pop;
use scale::layer1::room_quality::apply_room_quality_thoughts;
use scale::layer1::social_stratification::SocialClass;
use scale::layer1::terrain::{TerrainGrid, TerrainType};
use scale::layer1::zone::{ZoneGrid, ZoneType};
use scale::shared::time::SimulationTime;

fn setup_world() -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.insert_resource(ZoneGrid::new(10, 10));
    world.insert_resource(BeautyGrid::new(10, 10));
    let terrain = TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Grass; 100],
    };
    world.insert_resource(terrain);
    world.insert_resource(SimulationTime::default());
    world
}

#[test]
fn social_class_affects_room_expectations() {
    let mut world = setup_world();

    // 1. Create a "Decent" room at (0, 0)
    // Base 1.0 (space). Beauty needed ~20 to reach ~41 total quality.
    // Quality = (Space + Beauty * 2.0) * Enclosure(1.5 if enclosed)
    // Let's make it simpler: No enclosure (1.0). Space=1. Beauty=15.
    // Quality = (1 + 15 * 2) = 31.
    // This falls into "Decent" tier (25 <= Q < 50) for Labor.

    let mut zones = world.resource_mut::<ZoneGrid>();
    zones.set(0, 0, ZoneType::Bedroom);

    let mut beauty = world.resource_mut::<BeautyGrid>();
    beauty.set(0, 0, 15.0);

    // 2. Spawn Labor Pop
    let labor_pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            SocialClass::Labor,
            Memories::default(),
        ))
        .id();

    // 3. Spawn Elite Pop
    let elite_pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            SocialClass::Elite,
            Memories::default(),
        ))
        .id();

    // 4. Apply thoughts
    apply_room_quality_thoughts(&mut world, labor_pop, ZoneType::Bedroom);
    apply_room_quality_thoughts(&mut world, elite_pop, ZoneType::Bedroom);

    // 5. Assertions

    // Laborer should be happy with Decent room
    let labor_memories = world.get::<Memories>(labor_pop).unwrap();
    assert!(
        labor_memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::SleptInDecentRoom),
        "Laborer should consider quality 31 as Decent. Got: {:?}",
        labor_memories.items
    );

    // Elite should be unimpressed (Higher standards)
    // For Elite, "Decent" threshold should be higher (e.g., 75+).
    // So 31 quality should be "Dull" or "Awful".
    // Proposed thresholds: Awful < 40, Dull < 75.
    // So 31 is Awful for Elite.
    let elite_memories = world.get::<Memories>(elite_pop).unwrap();
    assert!(
        elite_memories.items.iter().any(|m| m.memory_type == MemoryType::SleptInAwfulRoom),
        "Elite should consider quality 31 as Awful (Standard Decent is Awful for them). Got: {:?}",
        elite_memories.items
    );
}
