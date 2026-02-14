use bevy_ecs::system::RunSystemOnce;
use scale::layer1::{
    AmbientLight, BuildingType, ColonyResources, GridPosition, LightMap, Pop, Speed, Tech,
    TechState, TerrainGrid, TerrainType, try_place_building,
};
use scale::setup::setup_world;

#[test]
fn test_tavern_emits_light() {
    let mut world = setup_world();

    // 1. Set Ambient Light to 0.0 (Pitch Black)
    world.resource_mut::<AmbientLight>().level = 0.0;

    // 2. Setup Terrain (ensure we can build)
    let mut terrain = world.resource_mut::<TerrainGrid>();
    terrain.tiles.fill(TerrainType::Grass);

    // 3. Setup Resources (ensure we can afford)
    world.resource_mut::<ColonyResources>().wood = 1000.0;
    world.resource_mut::<ColonyResources>().stone = 1000.0;

    // 4. Unlock Tech
    world
        .resource_mut::<TechState>()
        .unlock(Tech::SocialStructures);

    // 5. Place Tavern at (10, 10)
    let placed = try_place_building(&mut world, 10, 10, BuildingType::Tavern);
    assert!(placed.is_some(), "Should be able to place Tavern");

    // 6. Run Lighting Systems Manually (avoid DayNightCycle interference)
    world
        .run_system_once(scale::layer1::lighting::update_lighting_system)
        .unwrap();

    // 7. Check LightMap
    let light_map = world.resource::<LightMap>();
    let light_level = light_map.get(10, 10);
    assert!(
        light_level > 0.1,
        "Tavern should emit light, got {}",
        light_level
    );
}

#[test]
fn test_building_light_affects_pop_speed() {
    let mut world = setup_world();

    // 1. Set Ambient Light to 0.0 (Pitch Black)
    world.resource_mut::<AmbientLight>().level = 0.0;

    // 2. Setup Terrain (ensure we can build)
    let mut terrain = world.resource_mut::<TerrainGrid>();
    terrain.tiles.fill(TerrainType::Grass);

    // 3. Setup Resources (ensure we can afford)
    world.resource_mut::<ColonyResources>().wood = 1000.0;
    world.resource_mut::<ColonyResources>().stone = 1000.0;

    // 4. Unlock Tech
    world
        .resource_mut::<TechState>()
        .unlock(Tech::SocialStructures);

    // 5. Place Tavern at (10, 10)
    let placed = try_place_building(&mut world, 10, 10, BuildingType::Tavern);
    assert!(placed.is_some(), "Should be able to place Tavern");

    // 6. Spawn Pop near Tavern (11, 10) -> Lit
    let pop_lit = world
        .spawn((
            Pop,
            GridPosition { x: 11, y: 10 },
            Speed::default(),
            scale::layer1::Needs::default(),
        ))
        .id();

    // 7. Spawn Pop far away (0, 0) -> Dark
    let pop_dark = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Speed::default(),
            scale::layer1::Needs::default(),
        ))
        .id();

    // 8. Run Systems Manually
    world
        .run_system_once(scale::layer1::lighting::update_lighting_system)
        .unwrap();
    world
        .run_system_once(scale::layer1::lighting::apply_lighting_penalties_system)
        .unwrap();

    // 9. Check Speeds
    let speed_lit = world.get::<Speed>(pop_lit).unwrap().current;
    let speed_dark = world.get::<Speed>(pop_dark).unwrap().current;

    assert!(
        speed_lit > speed_dark,
        "Lit pop speed ({}) should be > Dark pop speed ({})",
        speed_lit,
        speed_dark
    );
    // Dark pop speed should be penalized (base * 0.5)
    // Lit pop speed should be base (1.0)
    assert!(
        (speed_lit - 1.0).abs() < 0.1,
        "Lit pop speed should be ~1.0, got {}",
        speed_lit
    );
    assert!(
        (speed_dark - 0.5).abs() < 0.1,
        "Dark pop speed should be ~0.5, got {}",
        speed_dark
    );
}
