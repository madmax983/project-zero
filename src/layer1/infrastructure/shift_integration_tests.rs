use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::farm::Farm;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::utility_ai::evaluate_actions_system;
    use crate::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};

    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        crate::setup::init_task_pools(); // Important for ComputeTaskPool
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10)); // Needed for evaluation
        world.insert_resource(crate::layer1::temperature::TemperatureGrid::new(
            10, 10, 20.0,
        )); // Needed for clothing eval
        world
    }

    #[test]
    fn test_refine_respects_shifts() {
        let mut world = setup_world();

        // 1. Setup Resources (Wood for Lumber Mill)
        world.resource_mut::<ColonyResources>().wood = 100.0;

        // 2. Setup Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(), // Not hungry/tired
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 100, // Ready to evaluate
                    ..Default::default()
                },
            ))
            .id();

        // 3. Setup Lumber Mill (Refining)
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
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // 4. Test Day (Should work)
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Day;
        }

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Refine,
            "Pop should work at Lumber Mill during day"
        );

        // Reset Pop
        world.entity_mut(pop).insert(PopAction {
            current: ActionType::Idle,
            ticks_committed: 100,
            ..Default::default()
        });

        // 5. Test Night (Should NOT work because night_shift is false)
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Pop should NOT work at Lumber Mill during night (shift closed)"
        );

        // 6. Enable Night Shift
        world.entity_mut(mill).insert(ShiftSchedule {
            day_shift: true,
            night_shift: true,
        });

        // Reset Pop
        world.entity_mut(pop).insert(PopAction {
            current: ActionType::Idle,
            ticks_committed: 100,
            ..Default::default()
        });

        // 7. Test Night Again (Should work now)
        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Refine,
            "Pop should work at Lumber Mill during night when shift is open"
        );
    }

    #[test]
    fn test_farm_respects_shifts() {
        let mut world = setup_world();

        // Setup Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 100,
                    ..Default::default()
                },
            ))
            .id();

        // Setup Farm
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
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Test Night (Closed)
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Pop should NOT farm at night (shift closed)"
        );

        // Enable Night Shift
        world.entity_mut(farm).insert(ShiftSchedule {
            day_shift: true,
            night_shift: true,
        });

        // Reset Pop
        world.entity_mut(pop).insert(PopAction {
            current: ActionType::Idle,
            ticks_committed: 100,
            ..Default::default()
        });

        // Test Night (Open)
        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Farm,
            "Pop should farm at night when shift is open"
        );
    }

    #[test]
    fn test_research_respects_shifts() {
        let mut world = setup_world();

        // Setup Pop (Intellectual helps research score, but default might be enough)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 100,
                    ..Default::default()
                },
            ))
            .id();

        // Setup Library
        let library = world
            .spawn((
                Building {
                    building_type: BuildingType::Library,
                },
                GridPosition { x: 2, y: 0 },
                crate::layer1::tech::Library,
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Test Night (Closed)
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Research,
            "Pop should NOT research at night (shift closed)"
        );

        // Enable Night Shift
        world.entity_mut(library).insert(ShiftSchedule {
            day_shift: true,
            night_shift: true,
        });

        // Reset Pop
        world.entity_mut(pop).insert(PopAction {
            current: ActionType::Idle,
            ticks_committed: 100,
            ..Default::default()
        });

        // Test Night (Open)
        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Research,
            "Pop should research at night when shift is open"
        );
    }