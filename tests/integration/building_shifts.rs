#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::GridPosition;
    use scale::layer1::building::{Building, BuildingType, ShiftSchedule};
    use scale::layer1::day_night::{DayNightCycle, TimeOfDay};
    use scale::layer1::farm::Farm;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{ColonyResources, RefiningProgress};
    use scale::layer1::tech::Library;
    use scale::layer1::utility_ai::evaluate_actions_system;
    use scale::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
    use scale::shared::time::SimulationTime;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        // Insert necessary resources for evaluate_actions_system
        world.insert_resource(scale::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(DayNightCycle::default());

        // Mock other resources used by queries in evaluate_actions_system
        world.insert_resource(scale::layer1::beauty::BeautyGrid::new(10, 10));
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));
        world.insert_resource(scale::layer1::acoustic::NoiseMap::new(10, 10));
        world.insert_resource(scale::layer1::lighting::LightMap::new(10, 10));
        world.insert_resource(scale::layer1::atmosphere::AtmosphereGrid::new(10, 10));
        world.insert_resource(scale::layer1::notifications::NotificationQueue::default());
        world.insert_resource(scale::layer1::trade::MerchantState::default());
        world.insert_resource(scale::layer1::vermin::VerminState::default());
        world.insert_resource(scale::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(scale::layer1::visitor::VisitorSource::default());
        world.insert_resource(scale::layer1::weather::WeatherState::default());
        world.insert_resource(scale::layer1::quirks::PlanetaryTraits::default());
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.insert_resource(scale::layer1::ColonyMemory::default());
        world.insert_resource(scale::layer1::erosion::ErosionGrid::new(10, 10));

        // Needed for entity spawning
        world.insert_resource(scale::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::shared::log::MessageLog::default());

        world
    }

    #[test]
    fn test_evaluate_refine_respects_shifts() {
        let mut world = setup_world();

        // 1. Setup Resources (Needs wood to refine)
        world.resource_mut::<ColonyResources>().wood = 10.0;

        // 2. Spawn Pop (Needs nothing, ready to work)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                scale::layer1::needs::Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 1.0,
                },
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.0,
                    ticks_committed: 10, // Ready to evaluate
                },
            ))
            .id();

        // 3. Spawn LumberMill with Day Shift Only
        let mill = world
            .spawn((
                Building {
                    building_type: BuildingType::LumberMill,
                },
                GridPosition { x: 5, y: 5 },
                RefiningProgress::default(),
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Case A: Day Time (Should Work)
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Day;
        }

        // Run evaluation
        world.run_system_once(evaluate_actions_system).unwrap();

        // Check result
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Refine,
            "Should refine during Day Shift"
        );

        // Case B: Night Time (Should NOT Work)
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        // Reset Pop
        world.get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;
        world.get_mut::<PopAction>(pop).unwrap().ticks_committed = 10;
        world.get_mut::<PopAction>(pop).unwrap().current_utility = 0.0;

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Should NOT refine during Night if Night Shift disabled"
        );

        // Case C: Enable Night Shift
        world.get_mut::<ShiftSchedule>(mill).unwrap().night_shift = true;

        // Reset Pop
        world.get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;
        world.get_mut::<PopAction>(pop).unwrap().ticks_committed = 10;
        world.get_mut::<PopAction>(pop).unwrap().current_utility = 0.0;

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Refine,
            "Should refine during Night if Night Shift enabled"
        );
    }

    #[test]
    fn test_evaluate_farm_respects_shifts() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                scale::layer1::needs::Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 1.0,
                },
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.0,
                    ticks_committed: 10,
                },
            ))
            .id();

        let _farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(), // Capacity 4
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Night Time, Night Shift Disabled
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        // Verify needs
        let needs = world.get::<scale::layer1::needs::Needs>(pop).unwrap();
        println!("Needs before eval: {:?}", needs);

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        println!(
            "Action picked: {:?} with utility {}",
            action.current, action.current_utility
        );
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Should not farm at night without shift"
        );
    }

    #[test]
    fn test_evaluate_research_respects_shifts() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                scale::layer1::needs::Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.0,
                    ticks_committed: 10,
                },
            ))
            .id();

        let _library = world
            .spawn((
                Building {
                    building_type: BuildingType::Library,
                },
                GridPosition { x: 5, y: 5 },
                Library,
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Night Time, Night Shift Disabled
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        world.run_system_once(evaluate_actions_system).unwrap();

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Should not research at night without shift"
        );
    }
}
