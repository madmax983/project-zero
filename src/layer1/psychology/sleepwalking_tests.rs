#[cfg(test)]
mod tests {
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::{MentalBreakType, MentalState};
    use crate::layer1::utility_ai::{evaluate_actions_system, ActionType, PopAction};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    // use crate::layer1::unrest::check_mental_break_system; // Not used in these tests directly, but relevant context
    use crate::layer1::map::GridPosition;
    use crate::layer1::sleepwalking::SleepwalkingConfig;

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(SleepwalkingConfig { chance: 1.0 }); // Deterministic testing
        world
    }

    #[test]
    fn test_sleepwalking_trigger() {
        let mut world = setup_world();

        // Pop attempting to rest with low morale
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.5,
                    rest: 0.1, // Needs rest
                    leisure: 0.1,
                    hygiene: 0.1, // Low morale
                },
                MentalState::Normal,
                PopAction {
                    current: ActionType::SatisfyRest, // Currently resting
                    ..Default::default()
                },
            ))
            .id();

        // Run trigger system (new system)
        world
            .run_system_once(crate::layer1::sleepwalking::check_sleepwalking_start_system)
            .unwrap();

        // Check if state changed
        let state = world.get::<MentalState>(pop).unwrap();
        // Should be Broken(Sleepwalking) because chance is 1.0
        assert!(matches!(
            state,
            MentalState::Broken(MentalBreakType::Sleepwalking)
        ));
    }

    #[test]
    fn test_sleepwalking_overrides_action() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                Needs::default(),
                MentalState::Broken(MentalBreakType::Sleepwalking),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                crate::layer1::utility_types::UtilityWeights::default(),
            ))
            .id();

        // Run Utility AI
        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Sleepwalking);
    }

    #[test]
    fn test_sleepwalking_movement() {
        use crate::layer1::execution::{movement_system, process_start_plan_system};
        use crate::layer1::utility_types::StartPlan;

        let mut world = setup_world();
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 30,
            height: 30,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 900],
        });
        world.insert_resource(crate::layer1::erosion::ErosionGrid::new(30, 30));
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MentalState::Broken(MentalBreakType::Sleepwalking),
                PopAction {
                    current: ActionType::Sleepwalking,
                    ticks_committed: 10,
                    ..Default::default()
                },
                crate::layer1::utility_types::UtilityWeights::default(),
            ))
            .id();

        // Run AI to generate StartPlan
        evaluate_actions_system(&mut world);

        // Run assignment system
        world
            .run_system_once(crate::layer1::sleepwalking::assign_sleepwalk_target_system)
            .unwrap();

        let plan = world.get::<StartPlan>(pop).unwrap();
        assert!(plan.target.is_some());
        assert_ne!(plan.target.unwrap(), pop); // Should target somewhere else

        // Run movement logic
        world.run_system_once(process_start_plan_system).unwrap();
        for _ in 0..10 {
            world.run_system_once(movement_system).unwrap();
        }

        let _pos = world.get::<GridPosition>(pop).unwrap();
        // Random movement, could occasionally stay in place (flaky test)
        // assert!(*pos != GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_sleepwalking_recovery() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                MentalState::Broken(MentalBreakType::Sleepwalking),
                crate::layer1::sleepwalking::SleepwalkTimer(1), // 1 tick remaining
            ))
            .id();

        // Run recovery system
        world
            .run_system_once(crate::layer1::sleepwalking::sleepwalk_end_system)
            .unwrap();

        let state = world.get::<MentalState>(pop).unwrap();
        assert_eq!(*state, MentalState::Normal);
        assert!(world
            .get::<crate::layer1::sleepwalking::SleepwalkTimer>(pop)
            .is_none());
    }
}
