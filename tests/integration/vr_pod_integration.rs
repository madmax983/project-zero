#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::artifacts::vr_pod::InVrPod;
    use scale::layer1::day_night::DayNightCycle;
    use scale::layer1::map::GridPosition;
    use scale::layer1::morale::Morale;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::stress::StressTracker;
    use scale::layer1::taboo::TabooState;
    use scale::layer1::utility_ai::{
        evaluate_actions_system, ActionType, PopAction, UtilityWeights,
    };
    use scale::layer1::utility_types::UtilityConfig;
    use scale::layer1::zone::ZoneGrid;
    use scale::shared::time::SimulationTime;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(DayNightCycle::default());
        world.insert_resource(TabooState::default());
        world.insert_resource(ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_in_vr_pod_ignores_utility_ai() {
        let mut world = setup_world();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1, // Very hungry! Normally would choose SatisfyHunger
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
                StressTracker::default(),
                Morale::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.0,
                    ticks_committed: 100, // Ready to switch
                },
                InVrPod, // The key component!
            ))
            .id();

        // Give them a farm to eat at
        world.spawn((
            scale::layer1::building::Building {
                building_type: scale::layer1::building::BuildingType::Farm,
            },
            GridPosition { x: 3, y: 0 },
            scale::layer1::farm::Farm::default(),
        ));

        // Evaluate AI
        evaluate_actions_system(&mut world);

        // Assert Pop is still Idle because InVrPod should make them skip AI evaluation
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Pop in VR Pod should not switch actions"
        );
    }
}
