use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::social::empty_room::{SanctuaryManager, Sanctuary};
use scale::layer1::stress::StressTracker;
use scale::layer1::utility_ai::{evaluate_actions_system, ActionType, PopAction};
use scale::layer1::zone::{ZoneGrid, ZoneType};
use scale::layer1::utility_types::UtilityConfig;
use scale::shared::time::SimulationTime;

fn setup_world() -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(UtilityConfig::default());

    // Setup ZoneGrid
    let mut zone_grid = ZoneGrid::new(10, 10);
    zone_grid.set(5, 5, ZoneType::Sanctuary);
    world.insert_resource(zone_grid);

    // Setup SanctuaryManager with a valid sanctuary
    let mut manager = SanctuaryManager::default();
    manager.sanctuaries.push(Sanctuary {
        is_valid: true,
        effectiveness: 1.0,
        tiles: vec![GridPosition { x: 5, y: 5 }],
    });
    world.insert_resource(manager);

    // Provide context needed by evaluating system
    world.insert_resource(scale::layer1::factions::Factions::default());
    world.insert_resource(scale::layer1::resources::ColonyResources::default());
    world.insert_resource(scale::layer1::weather::WeatherState::default());
    world.insert_resource(scale::layer1::seasons::SeasonState::default());
    world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
    world.insert_resource(scale::layer1::taboo::TabooState::default());
    world.insert_resource(scale::layer1::tech::TechState::default());

    // Terrain grid
    world.insert_resource(scale::layer1::terrain::TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
    });

    world
}

#[test]
fn test_stressed_pop_seeks_sanctuary() {
    let mut world = setup_world();

    // Spawn a highly stressed pop
    let pop = world
        .spawn((
            Pop,
            scale::layer1::needs::Needs::default(),
            scale::layer1::utility_types::UtilityWeights::default(),
            GridPosition { x: 0, y: 0 },
            StressTracker {
                accumulated_stress: 90.0, // High stress
                ..Default::default()
            },
            PopAction {
                ticks_committed: 100, // Make sure it's ready to evaluate
                ..Default::default()
            },
        ))
        .id();

    // Run evaluation system (this also populates the buffer)
    world.run_system_once(evaluate_actions_system).unwrap();

    // Verify the pop chose VisitSanctuary
    let action = world.get::<PopAction>(pop).unwrap();
    assert_eq!(
        action.current,
        ActionType::VisitSanctuary,
        "Highly stressed pop should choose VisitSanctuary"
    );
}

#[test]
fn test_unstressed_pop_ignores_sanctuary() {
    let mut world = setup_world();

    // Spawn an unstressed pop
    let pop = world
        .spawn((
            Pop,
            scale::layer1::needs::Needs::default(),
            scale::layer1::utility_types::UtilityWeights::default(),
            GridPosition { x: 0, y: 0 },
            StressTracker {
                accumulated_stress: 0.0, // No stress
                ..Default::default()
            },
            PopAction {
                ticks_committed: 100, // Make sure it's ready to evaluate
                ..Default::default()
            },
        ))
        .id();

    // Run evaluation system (this also populates the buffer)
    world.run_system_once(evaluate_actions_system).unwrap();

    // Verify the pop did NOT choose VisitSanctuary
    let action = world.get::<PopAction>(pop).unwrap();
    assert_ne!(
        action.current,
        ActionType::VisitSanctuary,
        "Unstressed pop should not choose VisitSanctuary"
    );
}
