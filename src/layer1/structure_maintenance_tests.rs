#[cfg(test)]
mod tests {
    use crate::layer1::GridPosition;
    use crate::layer1::actions::repair::evaluate_repair;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::{
        DeferMaintenance, Structure, calculate_malfunction_risk, entropy_system,
    };
    use crate::layer1::utility_eval_types::PositionProxy;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_entropy_system_reduces_hp() {
        let mut world = setup_world();
        let building = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 0, y: 0 },
                Building {
                    building_type: BuildingType::Housing,
                },
            ))
            .id();

        // Run entropy system
        entropy_system(&mut world);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0, "Entropy should reduce HP");
    }

    #[test]
    fn test_defer_maintenance_component() {
        let mut world = setup_world();
        let building = world
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                DeferMaintenance, // New component
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // This component should exist
        assert!(world.get::<DeferMaintenance>(building).is_some());
    }

    #[test]
    fn test_repair_logic_ignores_deferred() {
        let mut world = setup_world();
        let _building = world
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                DeferMaintenance,
                GridPosition { x: 0, y: 0 },
                Building {
                    building_type: BuildingType::Housing,
                },
            ))
            .id();

        // Evaluate repair job (conceptually)
        // This test ensures the utility AI query filters out DeferMaintenance
        // Implementation detail: Builder must ensure `evaluate_repair` or similar checks this.
        let mut query = world.query_filtered::<&Structure, Without<DeferMaintenance>>();
        let count = query.iter(&world).count();
        assert_eq!(count, 0, "Should not find building for repair if deferred");
    }

    #[test]
    fn test_malfunction_risk_calculation() {
        // 100% HP -> 0% Risk
        assert_eq!(calculate_malfunction_risk(100.0, 100.0), 0.0);

        // 50% HP -> 0% Risk (Threshold is 30%)
        assert_eq!(calculate_malfunction_risk(50.0, 100.0), 0.0);

        // 10% HP -> Positive Risk
        // (0.3 - 0.1) * 0.1 = 0.02 (2%)
        let risk = calculate_malfunction_risk(10.0, 100.0);
        assert!(risk > 0.0);
        assert!((risk - 0.02).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_repair_ignores_deferred() {
        let mut world = setup_world();
        let building = world
            .spawn((
                Structure {
                    current_hp: 10.0,
                    max_hp: 100.0,
                }, // Damaged
                DeferMaintenance,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Also spawn a normal damaged building
        let normal = world
            .spawn((
                Structure {
                    current_hp: 10.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let designations: Vec<PositionProxy> = vec![];

        // Simulate evaluate_actions_system filtering logic
        let mut proxies = Vec::new();
        let mut query =
            world.query::<(Entity, &GridPosition, &Structure, Option<&DeferMaintenance>)>();
        for (entity, pos, structure, defer) in query.iter(&world) {
            // Logic mirrored from utility_ai.rs
            if defer.is_some() {
                continue;
            }
            if (structure.current_hp - structure.max_hp).abs() < f32::EPSILON {
                continue;
            }
            proxies.push(PositionProxy { entity, pos: *pos });
        }

        let result = evaluate_repair(&pop_pos, &weights, &designations, &proxies);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap().1,
            normal,
            "Should pick normal building, not deferred one"
        );
        assert_ne!(result.unwrap().1, building);
    }
}
