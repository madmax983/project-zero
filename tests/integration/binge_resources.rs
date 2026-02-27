#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::execution::{arrival_handler_system, AtTarget, MovementTarget};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::unrest::{MentalBreakType, MentalState};
    use scale::layer1::utility_ai::{
        evaluate_actions_system, ActionType, PopAction, StartPlan, UtilityConfig,
    };
    use scale::shared::log::MessageLog;
    use scale::shared::time::SimulationTime;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup essential resources
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.init_resource::<Events<scale::layer1::items::UnequipEvent>>();

        // Initialize tasks pools for parallel queries
        scale::setup::init_task_pools();

        world
    }

    #[test]
    fn test_binge_finds_target() {
        let mut world = setup_world();

        // Spawn a pop with Binge mental break
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                MentalState::Broken(MentalBreakType::Binge),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 10, // Ready to evaluate
                    ..Default::default()
                },
                scale::layer1::utility_ai::UtilityWeights::default(),
            ))
            .id();

        // Spawn a stockpile nearby
        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                GridPosition { x: 5, y: 0 },
                Stockpile::default(),
            ))
            .id();

        // Run evaluation system
        evaluate_actions_system(&mut world);

        // Pop should have StartPlan for Binge action targeting the stockpile
        let plan = world.get::<StartPlan>(pop);
        assert!(plan.is_some(), "Pop should have a plan");
        let plan = plan.unwrap();

        assert_eq!(
            plan.action,
            ActionType::Binge,
            "Pop should choose Binge action"
        );
        assert_eq!(
            plan.target,
            Some(stockpile),
            "Pop should target the stockpile"
        );
    }

    #[test]
    fn test_binge_consumes_resources() {
        let mut world = setup_world();

        // Setup resources
        let mut resources = ColonyResources::default();
        resources.food = 20.0;
        resources.rations = 10.0;
        world.insert_resource(resources);

        // Spawn stockpile
        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                GridPosition { x: 5, y: 0 },
                Stockpile::default(),
            ))
            .id();

        // Spawn pop arriving at stockpile with Binge action
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 0 },
                Needs::default(),
                MovementTarget {
                    target_entity: stockpile,
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Binge,
                },
                AtTarget, // Simulate arrival
                MentalState::Broken(MentalBreakType::Binge),
            ))
            .id();

        // Run execution system
        // Note: we run arrival_handler_system directly or indirectly?
        // Let's run `arrival_handler_system` directly as in `execution.rs` tests.
        // It requires many queries, so running on world via Schedule is easier.
        let mut schedule = Schedule::default();
        schedule.add_systems(arrival_handler_system);
        schedule.run(&mut world);

        // Verify resources consumed
        let resources = world.resource::<ColonyResources>();
        // Should consume 5.0 food
        // 20.0 - 5.0 = 15.0
        assert!(
            (resources.food - 15.0).abs() < f32::EPSILON,
            "Food should decrease by 5.0 (was 20.0, is {})",
            resources.food
        );

        // Rations should be untouched (food sufficient)
        assert!(
            (resources.rations - 10.0).abs() < f32::EPSILON,
            "Rations should not decrease when food is sufficient"
        );

        // Verify log message
        let log = world.resource::<MessageLog>();
        assert!(!log.messages.is_empty());
        assert!(
            log.messages.iter().any(|m| m.text.contains("binge eating")),
            "Should log binge eating event"
        );

        // Verify movement components removed
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "MovementTarget should be removed"
        );
        assert!(
            world.get::<AtTarget>(pop).is_none(),
            "AtTarget should be removed"
        );
    }

    #[test]
    fn test_binge_spills_to_rations() {
        let mut world = setup_world();

        // Setup resources (low food)
        let mut resources = ColonyResources::default();
        resources.food = 2.0;
        resources.rations = 10.0;
        world.insert_resource(resources);

        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                GridPosition { x: 5, y: 0 },
                Stockpile::default(),
            ))
            .id();

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 0 },
            MovementTarget {
                target_entity: stockpile,
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Binge,
            },
            AtTarget,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(arrival_handler_system);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        // Food should be 0 (consumed 2.0)
        assert!(resources.food < f32::EPSILON);
        // Rations should be decreased by remaining 3.0 (10.0 - 3.0 = 7.0)
        assert!(
            (resources.rations - 7.0).abs() < f32::EPSILON,
            "Rations should decrease by remainder (was 10.0, is {})",
            resources.rations
        );
    }
}
