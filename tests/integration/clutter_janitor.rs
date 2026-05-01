#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::{
        clutter::ClutterGrid,
        map::GridPosition,
        pop::{Job, Pop},
        utility_types::{ActionType, AssignmentType, PopAction},
        needs::Needs,
        mind::utility_ai::UtilityConfig,
    };
    use scale::layer1::mind::utility_ai::evaluate_actions_system;

    #[test]
    fn test_janitor_prioritizes_cleaning() {
        let mut world = scale::setup::setup_world();
        world.insert_resource(ClutterGrid::new(10, 10));

        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(5, 5, 40.0); // Not critical, but high enough for Janitor

        let building = world.spawn(scale::layer1::building::Building {
            kind: scale::layer1::building::BuildingType::Wall,
        }).insert(GridPosition { x: 5, y: 5 }).id();
        world.resource_mut::<scale::layer1::building::BuildingMap>().0.insert((5, 5), building);

        let janitor_entity = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs::default(),
            PopAction::default(),
            Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::Janitor },
            scale::layer1::utility_types::UtilityWeights::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_actions_system);
        schedule.run(&mut world);

        let action = world.get::<PopAction>(janitor_entity).unwrap();
        assert_eq!(action.current, ActionType::Clean);
    }
}
