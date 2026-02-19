#[cfg(test)]
mod tests {
    use crate::layer1::actions::work::evaluate_work;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use crate::layer1::utility_eval_types::PositionProxy;
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

        let proxies: Vec<PositionProxy> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| PositionProxy { entity: e, pos: *p })
            .collect();

        let result = evaluate_work(pop_pos, &weights, &proxies);

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

        let proxies: Vec<PositionProxy> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| PositionProxy { entity: e, pos: *p })
            .collect();

        let (_, target) = evaluate_work(pop_pos, &weights, &proxies).unwrap();
        assert_eq!(target, close);
    }

    #[test]
    fn test_evaluate_work_no_designations() {
        let _world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let proxies: Vec<PositionProxy> = vec![];

        let result = evaluate_work(pop_pos, &weights, &proxies);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_work_success_modifier() {
        // Setup
        let pop_pos = GridPosition { x: 0, y: 0 };
        let mut weights = UtilityWeights::default();
        let idx = ActionType::Work.as_index();

        let proxies = vec![PositionProxy {
            entity: Entity::PLACEHOLDER,
            pos: GridPosition { x: 0, y: 0 },
        }];

        // Case 1: Neutral (No history) -> Utility ~0.5
        let (base_u, _) = evaluate_work(pop_pos, &weights, &proxies).unwrap();
        assert!(
            (base_u - 0.5).abs() < 0.01,
            "Base utility should be ~0.5, got {}",
            base_u
        );

        // Case 2: High Success -> Higher Utility
        weights.action_attempt_count[idx] = 10;
        weights.action_success_count[idx] = 10;
        let (high_u, _) = evaluate_work(pop_pos, &weights, &proxies).unwrap();
        assert!(
            high_u > base_u,
            "Success history should increase utility ({} > {})",
            high_u,
            base_u
        );

        // Case 3: High Failure -> Lower Utility
        weights.action_success_count[idx] = 2; // 20% success
        let (low_u, _) = evaluate_work(pop_pos, &weights, &proxies).unwrap();
        assert!(
            low_u < base_u,
            "Failure history should decrease utility ({} < {})",
            low_u,
            base_u
        );
    }

    #[test]
    fn test_evaluate_work_distance_scaling() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default(); // distance_weight = 1.0

        // Formula: 1.0 / (1.0 + 0.1 * dist)
        let cases = vec![
            (0, 1.0),   // 1.0 / 1.0 = 1.0
            (10, 0.5),  // 1.0 / 2.0 = 0.5
            (20, 0.333), // 1.0 / 3.0 = 0.333
            (90, 0.1),  // 1.0 / 10.0 = 0.1
        ];

        for (dist, expected_factor) in cases {
            let proxies = vec![PositionProxy {
                entity: Entity::PLACEHOLDER,
                pos: GridPosition { x: dist, y: 0 },
            }];

            let (u, _) = evaluate_work(pop_pos, &weights, &proxies).unwrap();
            let expected_u = 0.5 * expected_factor; // Base utility * factor
            assert!(
                (u - expected_u).abs() < 0.01,
                "Failed at distance {}: expected {}, got {}",
                dist,
                expected_u,
                u
            );
        }
    }

    #[test]
    fn test_evaluate_work_base_utility() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let proxies = vec![PositionProxy {
            entity: Entity::PLACEHOLDER,
            pos: GridPosition { x: 0, y: 0 },
        }];

        let (u, _) = evaluate_work(pop_pos, &weights, &proxies).unwrap();
        assert!(
            (u - 0.5).abs() < f32::EPSILON,
            "Base utility MUST be exactly 0.5 at distance 0 with neutral weights"
        );
    }
}
