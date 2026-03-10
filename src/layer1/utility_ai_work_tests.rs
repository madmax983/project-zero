#[cfg(test)]
mod tests {
    use crate::layer1::actions::evaluate_simple_action;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use bevy_ecs::prelude::*;

    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_noble_refuses_work() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        let proxies: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| ScorableCandidate::new(e, *p))
            .collect();

        // This evaluates generic base utility. The rejection logic resides in the decider.
        // We will test if evaluate_single_pop or similar avoids giving work to a noble.

        let noble = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            crate::layer1::needs::Needs::default(),
            UtilityWeights::default(),
            crate::layer1::utility_ai::PopAction::default(),
            Traits(std::collections::HashSet::from([Trait::Noble])),
        )).id();

        let mut buffer = crate::layer1::utility_eval_types::UtilityAIBuffer::default();
        buffer.work_designations = proxies.clone();

        // Setup pop eval data with Noble trait
        let mut data = crate::layer1::utility_eval_types::PopEvalData::test_instance();
        data.entity = noble;
        data.pos = pop_pos;
        data.traits = Some(Traits(std::collections::HashSet::from([Trait::Noble])));

        let context = crate::layer1::utility_eval_types::WorldContext {
            resources: &crate::layer1::resources::ColonyResources::default(),
            cycle: &crate::layer1::day_night::DayNightCycle::default(),
            taboo: &crate::layer1::taboo::TabooState::default(),
            factions: None,
            zone_grid: &crate::layer1::zone::ZoneGrid::new(10, 10),
            temperature_grid: None,
        };

        // We run evaluate_single_pop, which calls run() on PopDecider
        let (action, _, _) = crate::layer1::utility_ai::evaluate_single_pop(&buffer, &data, &context);

        assert_ne!(action, ActionType::Work, "Noble should refuse work");
    }

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

        let proxies: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| ScorableCandidate::new(e, *p))
            .collect();

        let result = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5);

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

        let proxies: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &Designation)>()
            .iter(&world)
            .map(|(e, p, _)| ScorableCandidate::new(e, *p))
            .collect();

        let (_, target) = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5).unwrap();
        assert_eq!(target, close);
    }

    #[test]
    fn test_evaluate_work_no_designations() {
        let _world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let proxies: Vec<ScorableCandidate> = vec![];

        let result = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_work_distance_scaling() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default(); // distance_weight = 1.0

        // Formula: 1.0 / (1.0 + 0.1 * dist)
        let cases = vec![
            (0, 1.0),    // 1.0 / 1.0 = 1.0
            (10, 0.5),   // 1.0 / 2.0 = 0.5
            (20, 0.333), // 1.0 / 3.0 = 0.333
            (90, 0.1),   // 1.0 / 10.0 = 0.1
        ];

        for (dist, expected_factor) in cases {
            let proxies = vec![ScorableCandidate::new(
                Entity::PLACEHOLDER,
                GridPosition { x: dist, y: 0 },
            )];

            let (u, _) = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5).unwrap();
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

        let proxies = vec![ScorableCandidate::new(
            Entity::PLACEHOLDER,
            GridPosition { x: 0, y: 0 },
        )];

        let (u, _) = evaluate_simple_action(pop_pos, &weights, &proxies, 0.5).unwrap();
        assert!(
            (u - 0.5).abs() < f32::EPSILON,
            "Base utility MUST be exactly 0.5 at distance 0 with neutral weights"
        );
    }
}
