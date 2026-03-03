use bevy_ecs::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::social::empty_room::{
    update_sanctuary_system, SanctuaryManager,
};
use scale::layer1::stress::StressTracker;
use scale::layer1::utility_ai::{evaluate_actions_system, ActionType, PopAction};
use scale::layer1::zone::{ZoneGrid, ZoneType};
use scale::layer1::needs::Needs;
use scale::layer1::utility_types::{UtilityWeights, UtilityConfig};

#[test]
fn test_stressed_pop_visits_sanctuary() {
    let mut world = World::new();

    // Init required resources
    world.insert_resource(UtilityConfig::default());
    world.insert_resource(scale::layer1::resources::ColonyResources::default());
    scale::setup::init_task_pools();

    // Setup ZoneGrid with Sanctuary at (5,5)
    let mut zone_grid = ZoneGrid::new(10, 10);
    zone_grid.set(5, 5, ZoneType::Sanctuary);
    world.insert_resource(zone_grid);
    world.insert_resource(SanctuaryManager::default());

    // Give it a day/night cycle to satisfy populate_farms/etc
    world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
    world.insert_resource(scale::layer1::taboo::TabooState::default());

    // Evaluate needs a SimulationTime resource
    world.insert_resource(scale::shared::time::SimulationTime::default());

    // Setup highly stressed Pop
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs::default(), // Other needs start fully satisfied (1.0), so urgency is low
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.0,
                ticks_committed: 1,
            },
            StressTracker {
                accumulated_stress: 80.0, // High stress (> 0.2 threshold)
                ..Default::default()
            },
        ))
        .id();

    // 1. Run update_sanctuary_system to identify the sanctuary
    let mut schedule = Schedule::default();
    schedule.add_systems(update_sanctuary_system);
    schedule.run(&mut world);

    let manager = world.resource::<SanctuaryManager>();
    assert_eq!(manager.sanctuaries.len(), 1, "Sanctuary should be registered");
    assert!(manager.sanctuaries[0].is_valid, "Sanctuary should be valid");

    // 2. Run Utility AI evaluate_actions_system
    let mut schedule = Schedule::default();
    schedule.add_systems(evaluate_actions_system);
    schedule.run(&mut world);

    // Assert the pop chose to visit the sanctuary
    let action = world.get::<scale::layer1::utility_types::StartPlan>(pop);
    assert!(action.is_some(), "Pop should have planned an action");
    assert_eq!(
        action.unwrap().action,
        ActionType::VisitSanctuary,
        "Pop should target the sanctuary due to high stress"
    );
}
