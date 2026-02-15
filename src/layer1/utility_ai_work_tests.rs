#[cfg(test)]
mod tests {
    use crate::layer1::actions::work::evaluate_work;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use crate::layer1::utility_eval_types::WorkDesignationProxy;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_action_type_work_variant() {
        let work = ActionType::Work;
        assert_eq!(work, ActionType::Work);
    }

    #[test]
    fn test_evaluate_work_finds_designation() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn a designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        let proxies: Vec<WorkDesignationProxy> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| WorkDesignationProxy { entity: e, pos: *p })
            .collect();

        let result = evaluate_work(&pop_pos, &weights, &proxies);

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, designation);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_work_prioritizes_closest() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 2.0, // High preference for close work
            ..Default::default()
        };

        // Far designation
        world.spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 20, y: 0 },
        ));

        // Close designation
        let close = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 2, y: 0 },
            ))
            .id();

        let proxies: Vec<WorkDesignationProxy> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| WorkDesignationProxy { entity: e, pos: *p })
            .collect();

        let (_, target) = evaluate_work(&pop_pos, &weights, &proxies).unwrap();
        assert_eq!(target, close);
    }

    #[test]
    fn test_evaluate_work_no_designations() {
        let _world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let proxies: Vec<WorkDesignationProxy> = vec![];

        let result = evaluate_work(&pop_pos, &weights, &proxies);
        assert!(result.is_none());
    }
}
