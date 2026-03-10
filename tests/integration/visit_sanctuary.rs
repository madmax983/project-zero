use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use scale::layer1::clutter::ClutterGrid;
use scale::layer1::execution::arrival::arrival_handler_system;
use scale::layer1::execution::movement::movement_system;
use scale::layer1::execution::process_start_plan_system;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::social::empty_room::{
    update_sanctuary_system, visit_sanctuary_system, SanctuaryManager,
};
use scale::layer1::stress::StressTracker;
use scale::layer1::utility_ai::{evaluate_actions_system, update_action_timer_system};
use scale::layer1::utility_types::{ActionType, PopAction, UtilityConfig};
use scale::layer1::zone::{ZoneGrid, ZoneType};
use scale::setup::init_task_pools;

fn setup_app() -> App {
    init_task_pools();
    let mut app = App::new();

    // Grids
    let mut zone_grid = ZoneGrid::new(10, 10);
    zone_grid.set(5, 5, ZoneType::Sanctuary); // A 1-tile sanctuary at (5,5)
    app.insert_resource(zone_grid);
    app.insert_resource(ClutterGrid::new(10, 10));
    app.insert_resource(SanctuaryManager::default());
    app.insert_resource(UtilityConfig::default());
    app.insert_resource(scale::layer1::resources::ColonyResources::default());
    app.insert_resource(scale::layer1::temperature::TemperatureGrid::new(
        10, 10, 20.0,
    ));
    app.insert_resource(scale::layer1::day_night::DayNightCycle::default());
    app.insert_resource(scale::layer1::taboo::TabooState::default());

    // Need a terrain grid for movement_system
    let mut terrain = scale::layer1::terrain::generate_terrain(10, 10);
    for x in 0..10 {
        for y in 0..10 {
            terrain.set(x, y, scale::layer1::terrain::TerrainType::Grass);
        }
    }
    app.insert_resource(terrain);
    app.init_resource::<scale::shared::time::SimulationTime>();
    app.init_resource::<scale::layer1::weather::WeatherState>();
    app.insert_resource(scale::layer1::erosion::ErosionGrid::new(10, 10));
    app.init_resource::<bevy_ecs::event::Events<scale::layer1::items::UnequipEvent>>();

    // Systems
    app.add_systems(
        Update,
        (
            update_action_timer_system,
            update_sanctuary_system,
            evaluate_actions_system,
            process_start_plan_system,
            movement_system,
            arrival_handler_system,
            visit_sanctuary_system,
        )
            .chain(),
    );

    app
}

#[test]
fn test_stressed_pop_visits_sanctuary() {
    let mut app = setup_app();

    // Spawn a highly stressed pop
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            StressTracker {
                accumulated_stress: 90.0, // High stress
                ..Default::default()
            },
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.0,
                ticks_committed: 1000,
            },
            scale::layer1::needs::Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 1.0,
            }, // Other needs satisfied
            scale::layer1::utility_types::UtilityWeights::default(),
            scale::layer1::pop::Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        ))
        .id();

    // 1. Run the simulation to update sanctuaries and evaluate actions
    app.update();

    // Check if the pop decided to visit the sanctuary
    let action = app.world().get::<PopAction>(pop).unwrap();
    if action.current != ActionType::VisitSanctuary {
        panic!(
            "Expected VisitSanctuary, but got {:?} (utility: {})",
            action.current, action.current_utility
        );
    }

    // 2. Step the simulation to allow the pop to move to the target.
    // The pop starts at (0,0) and needs to get to (5,5), which takes 10 moves (manhattan distance).
    for _ in 0..25 {
        app.world_mut()
            .resource_mut::<scale::shared::time::SimulationTime>()
            .tick += 1;
        app.update();
    }

    // Ensure they arrived
    let pos = app.world().get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 5);
    assert_eq!(pos.y, 5);

    // 3. Check stress reduction
    let stress = app
        .world()
        .get::<StressTracker>(pop)
        .unwrap()
        .accumulated_stress;
    assert!(
        stress < 90.0,
        "Stress should be reduced after visiting the sanctuary"
    );
}
