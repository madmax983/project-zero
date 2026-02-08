#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::designation::{Designation, DesignationType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::memory::{Memories, MemoryType};
    use scale::layer1::utility_ai::{ActionType, PopAction, UtilityConfig, UtilityWeights, evaluate_actions_system};
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
    fn test_low_morale_reduces_work_utility() {
        let mut world = setup_world();

        // Work designation
        let _designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 0 },
        )).id();

        // Happy Pop
        let happy_pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { leisure: 1.0, hunger: 1.0, rest: 1.0 }, // Max needs morale
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 100,
            },
            Memories::default(), // No bad memories
        )).id();

        // Depressed Pop
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);
        memories.add(MemoryType::WitnessedDeath, 0); // Stacking penalty

        let depressed_pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { leisure: 1.0, hunger: 1.0, rest: 1.0 }, // Good physical needs
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.1,
                ticks_committed: 100,
            },
            memories,
        )).id();

        // Evaluate
        world.run_system_once(evaluate_actions_system).unwrap();

        // Both should choose Work because needs are met
        let happy_action = world.get::<PopAction>(happy_pop).unwrap();
        let depressed_action = world.get::<PopAction>(depressed_pop).unwrap();

        assert_eq!(happy_action.current, ActionType::Work, "Happy pop should work");
        assert_eq!(depressed_action.current, ActionType::Work, "Depressed pop should work (best option)");

        // Currently, effective morale is ignored, so utilities will be equal.
        // After integration, happy utility should be higher.
        assert!(
            happy_action.current_utility > depressed_action.current_utility,
            "Happy pop utility {} should be > Depressed pop utility {}",
            happy_action.current_utility,
            depressed_action.current_utility
        );
    }
}
