use scale::layer1::{
    AmbientLight, BuildingType, ColonyResources, GridPosition, LightMap, Pop, Speed, Tech,
    TechState, TerrainGrid, TerrainType, try_place_building,
};
use scale::setup::setup_world;
use scale::simulation::run_simulation_tick;

#[test]
fn test_tavern_emits_light() {
    let mut world = setup_world();

    // 1. Set Ambient Light to 0.0 (Pitch Black) by setting Night
    // We must set the tick to Night time, otherwise update_day_night_cycle_system will reset it to Dawn/Day
    {
        let mut time = world.resource_mut::<scale::shared::time::SimulationTime>();
        // Default ticks_per_day is 250. Night starts at 0.85 * 250 = 212.5
        time.tick = 220;
    }

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
    assert!(placed, "Should be able to place Tavern");

    // 6. Run Tick (updates lighting)
    run_simulation_tick(&mut world);

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

    // 1. Set Ambient Light to 0.0 (Pitch Black) by setting Night
    {
        let mut time = world.resource_mut::<scale::shared::time::SimulationTime>();
        time.tick = 220;
    }

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
    assert!(placed, "Should be able to place Tavern");

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

    // 8. Run Tick
    run_simulation_tick(&mut world);

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
