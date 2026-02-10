#[cfg(test)]
mod tests {
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::movement::{AtTarget, MovementTarget};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{Carrying, ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
    use scale::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_hauling_end_to_end() {
        let mut world = World::new();
        // Setup core resources
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(scale::layer1::utility_ai::types::UtilityConfig::default());
        scale::setup::init_task_pools();

        // 1. Spawn a Pop at (0,0)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                PopAction {
                    ticks_committed: 100, // Force evaluation
                    ..Default::default()
                },
                UtilityWeights::default(),
            ))
            .id();

        // 2. Spawn an Item at (5,0)
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 10.0,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // 3. Spawn a Stockpile at (10,0)
        let _stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // 4. Run AI Evaluation (should pick Haul)
        world.run_system_once(scale::layer1::utility_ai::update_action_timer_system).unwrap();
        scale::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Haul,
            "Pop should decide to Haul"
        );

        // 5. Process Plan -> Movement
        world.run_system_once(scale::layer1::execution::cleanup_previous_assignment_system).unwrap();
        world.run_system_once(scale::layer1::movement::process_start_plan_system).unwrap();

        // Should have MovementTarget to Item
        let mt = world.get::<MovementTarget>(pop).unwrap();
        assert_eq!(mt.target_entity, item, "Should target item first");

        // 6. Simulate Movement to Item
        // For test speed, we just teleport and add AtTarget
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 5, y: 0 };
        world.entity_mut(pop).insert(AtTarget);

        // 7. Execute Haul (Pickup)
        scale::layer1::hauling::haul_system(&mut world);

        // Pop should be carrying
        assert!(world.get::<Carrying>(pop).is_some());
        // Item entity should be gone
        assert!(world.get_entity(item).is_err());
        // MovementTarget should be gone (system cleared it)
        assert!(world.get::<MovementTarget>(pop).is_none());

        // 8. Run AI Evaluation Again (should continue Haul - phase 2)
        // Actually, haul_system handles the phase transition logic if we are running it every tick?
        // But haul_system cleared MovementTarget.
        // So evaluate_actions_system needs to see we are carrying and target stockpile.
        // Let's reset action commit timer to force re-eval.
        world.get_mut::<PopAction>(pop).unwrap().ticks_committed = 10;

        scale::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Haul,
            "Pop should continue Hauling"
        );

        // 9. Process Plan -> Movement (to Stockpile)
        world.run_system_once(scale::layer1::execution::cleanup_previous_assignment_system).unwrap();
        world.run_system_once(scale::layer1::movement::process_start_plan_system).unwrap();

        let mt = world.get::<MovementTarget>(pop).unwrap();
        // Target should be stockpile entity (found by evaluate_haul)
        // We didn't capture stockpile ID, but we can verify position
        assert_eq!(mt.target_position, GridPosition { x: 10, y: 0 });

        // 10. Simulate Movement to Stockpile
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 10, y: 0 };
        world.entity_mut(pop).insert(AtTarget);

        // 11. Execute Haul (Drop)
        scale::layer1::hauling::haul_system(&mut world);

        // Resources should be updated
        let res = world.resource::<ColonyResources>();
        assert!((res.wood - 110.0).abs() < f32::EPSILON); // Default 100 + 10
    }
}
