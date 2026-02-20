#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{Carrying, ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};
    use scale::layer1::utility_ai::evaluate_actions_system;
    use scale::shared::time::SimulationTime;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());

        let mut resources = ColonyResources::default();
        resources.tools = 0.0; // Prevent FetchTool action
        world.insert_resource(resources);

        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world
    }

    #[test]
    fn test_hauling_pop_targets_stockpile_not_item() {
        let mut world = setup_world();

        // 1. Spawn Stockpile at (10, 0)
        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // 2. Spawn loose Wood item at (5, 0)
        let loose_item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 1.0,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // 3. Spawn Pop at (0, 0) carrying Wood
        // The pop is carrying wood, so it should want to drop it off at the stockpile.
        // It should NOT go pick up the loose wood item.
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(), // Full needs, so Idle or Haul
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 100, // Ready to evaluate
                    ..Default::default()
                },
                Carrying {
                    resource_type: ResourceType::Wood,
                    amount: 1.0,
                },
            ))
            .id();

        // 4. Run Evaluation
        world.run_system_once(evaluate_actions_system).unwrap();

        // 5. Check Result
        let action = world.get::<PopAction>(pop).unwrap();

        // Current behavior (bug): Pop targets loose_item because evaluate_haul ignores Carrying
        // Expected behavior (fix): Pop targets stockpile because it is carrying
        assert_eq!(action.current, ActionType::Haul, "Pop should be hauling");

        let start_plan = world.get::<scale::layer1::utility_types::StartPlan>(pop);
        assert!(start_plan.is_some(), "Should have a start plan");
        let target = start_plan.unwrap().target;

        assert_eq!(
            target,
            Some(stockpile),
            "Pop should target stockpile when carrying, but targeted {:?} (likely loose item {:?})",
            target,
            loose_item
        );
    }
}
