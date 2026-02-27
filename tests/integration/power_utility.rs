#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::energy::PowerConsumer;
    use scale::layer1::farm::Farm;
    use scale::layer1::health::Health;
    use scale::layer1::map::GridPosition;
    use scale::layer1::medical::Hospital;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{ColonyResources, RefiningProgress};
    use scale::layer1::utility_ai::{
        evaluate_actions_system, update_action_timer_system, ActionType, PopAction, UtilityConfig,
        UtilityWeights,
    };
    use scale::shared::time::SimulationTime;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_unpowered_smelter_ignored() {
        let mut world = setup_world();

        // Add resources for Smelting (Ore + Wood)
        let mut resources = ColonyResources::default();
        resources.ore = 10.0;
        resources.wood = 10.0;
        world.insert_resource(resources);

        // Spawn Unpowered Smelter
        let _smelter = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
                RefiningProgress::default(),
                PowerConsumer {
                    demand: 5.0,
                    active: false, // Unpowered
                },
            ))
            .id();

        // Spawn Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Run AI
        let mut schedule = Schedule::default();
        schedule.add_systems((update_action_timer_system, evaluate_actions_system).chain());
        schedule.run(&mut world);

        // Assert Pop did NOT choose Refine
        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Refine,
            "Pop should not choose Refine at unpowered Smelter"
        );
    }

    #[test]
    fn test_unpowered_hydroponics_ignored() {
        let mut world = setup_world();

        // Spawn Unpowered Hydroponics
        let _farm = world
            .spawn((
                Building {
                    building_type: BuildingType::HydroponicsBay,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
                PowerConsumer {
                    demand: 5.0,
                    active: false, // Unpowered
                },
            ))
            .id();

        // Spawn Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Run AI
        let mut schedule = Schedule::default();
        schedule.add_systems((update_action_timer_system, evaluate_actions_system).chain());
        schedule.run(&mut world);

        // Assert Pop did NOT choose Farm
        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Farm,
            "Pop should not choose Farm at unpowered Hydroponics"
        );
    }

    #[test]
    fn test_unpowered_hospital_ignored() {
        let mut world = setup_world();

        // Spawn Unpowered Hospital
        let _hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                GridPosition { x: 5, y: 5 },
                Hospital::default(),
                PowerConsumer {
                    demand: 5.0,
                    active: false, // Unpowered
                },
            ))
            .id();

        // Spawn Sick Pop (needs medical care)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
                Health {
                    current: 50.0, // Damaged
                    max: 100.0,
                },
            ))
            .id();

        // Run AI
        let mut schedule = Schedule::default();
        schedule.add_systems((update_action_timer_system, evaluate_actions_system).chain());
        schedule.run(&mut world);

        // Assert Pop did NOT choose SeekMedicalCare
        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::SeekMedicalCare,
            "Pop should not choose SeekMedicalCare at unpowered Hospital"
        );
    }
}
