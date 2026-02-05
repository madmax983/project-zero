#[cfg(test)]
mod tests {
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::{GridPosition, Pop};
    use crate::layer1::utility_ai::{
        ActionType, UtilityWeights, calculate_context_score, evaluate_work,
    };
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

        let mut designations = world.query::<(Entity, &GridPosition, &Designation)>();

        let result = evaluate_work(&pop_pos, &weights, designations.iter(&world));

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

        let mut designations = world.query::<(Entity, &GridPosition, &Designation)>();

        let (_, target) = evaluate_work(&pop_pos, &weights, designations.iter(&world)).unwrap();
        assert_eq!(target, close);
    }

    #[test]
    fn test_evaluate_work_no_designations() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let mut designations = world.query::<(Entity, &GridPosition, &Designation)>();

        let result = evaluate_work(&pop_pos, &weights, designations.iter(&world));
        assert!(result.is_none());
    }
}
