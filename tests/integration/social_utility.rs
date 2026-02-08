#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::social::{Relationships, Tavern};
    use scale::layer1::utility_ai::{ActionType, PopAction, UtilityConfig, UtilityWeights, evaluate_actions_system, StartPlan};
    use scale::layer1::resources::ColonyResources;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_pop_prefers_tavern_with_friends() {
        let mut world = setup_world();

        // Spawn Tavern A at (5, 0)
        let tavern_a = world.spawn((
            Building { building_type: BuildingType::Tavern },
            GridPosition { x: 5, y: 0 },
            Tavern::default(),
        )).id();

        // Spawn Tavern B at (5, 0) - same location to eliminate distance bias
        let tavern_b = world.spawn((
            Building { building_type: BuildingType::Tavern },
            GridPosition { x: 5, y: 0 },
            Tavern::default(),
        )).id();

        // Friend
        let friend = world.spawn(Pop).id();
        // Stranger
        let stranger = world.spawn(Pop).id();

        // Put Stranger in Tavern A
        world.get_mut::<Tavern>(tavern_a).unwrap().visitors.push(stranger);
        // Put Friend in Tavern B
        world.get_mut::<Tavern>(tavern_b).unwrap().visitors.push(friend);

        // Subject Pop
        // Needs high leisure need to trigger socialize
        let subject = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { leisure: 0.1, hunger: 1.0, rest: 1.0 }, // 0.1 leisure = high urgency
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 100, // Ensure evaluation runs
            },
            Relationships::with_affinity(friend, 50.0), // Likes friend
        )).id();

        // Evaluate
        world.run_system_once(evaluate_actions_system).unwrap();

        // Check StartPlan
        let plan = world.get::<StartPlan>(subject);
        assert!(plan.is_some(), "Subject should have a plan");
        let plan = plan.unwrap();
        assert_eq!(plan.action, ActionType::Socialize);

        assert_eq!(plan.target, Some(tavern_b), "Subject should choose Tavern B (with friend)");
    }
}
