use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
    use crate::layer1::day_night::DayNightCycle;
    use crate::layer1::farm::Farm;
    use crate::layer1::map::GridPosition;

    use crate::layer1::resources::{ColonyResources, RefiningProgress};

    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;


    // Helper to evaluate refine
    use crate::layer1::actions::evaluate_simple_action;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(DayNightCycle::default());
        world
    }

    #[test]
    fn test_evaluate_refine_finds_valid_work() {
        let mut world = setup_world();

        // Add resources for input (Wood for Lumber Mill)
        world.resource_mut::<ColonyResources>().wood = 10.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Lumber Mill
        let mill = world
            .spawn((
                Building {
                    building_type: BuildingType::LumberMill,
                },
                GridPosition { x: 2, y: 0 },
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule::default(),
            ))
            .id();

        // Create Proxy
        let proxies = vec![ScorableCandidate::new(mill, GridPosition { x: 2, y: 0 })];

        let result = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5);

        assert!(result.is_some());
        let (utility, target) = result.expect("Missing resource or component");
        assert_eq!(target, mill);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_farm_finds_work() {
        let mut world = setup_world();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Farm
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 2, y: 0 },
                Farm {
                    capacity: 1,
                    workers: vec![],
                    ..Default::default()
                },
                ShiftSchedule::default(),
            ))
            .id();

        // Create Proxy
        let proxies = vec![ScorableCandidate::with_capacity(
            farm,
            GridPosition { x: 2, y: 0 },
            1,
            0,
        )];

        let result = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5);

        assert!(result.is_some());
        let (utility, target) = result.expect("Missing resource or component");
        assert_eq!(target, farm);
        assert!(utility > 0.0);
    }