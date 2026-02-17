#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::edicts::ColonyPolicies;
    use scale::layer1::map::GridPosition;
    use scale::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::vermin::{VerminState, vermin_growth_system};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(VerminState::default());
        // Start with zero stored resources to isolate item effect
        world.insert_resource(ColonyResources::zeroed());
        world.insert_resource(ColonyPolicies::default());
        world
    }

    #[test]
    fn test_vermin_growth_from_food_items_on_ground() {
        let mut world = setup_world();

        // Spawn food item on ground
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Food,
                amount: 1000.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(
            vermin.severity > 0.0,
            "Vermin should grow from food items on ground"
        );
    }

    #[test]
    fn test_vermin_growth_from_waste_items_on_ground() {
        let mut world = setup_world();

        // Spawn waste item on ground
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Waste,
                amount: 500.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(
            vermin.severity > 0.0,
            "Vermin should grow from waste items on ground"
        );
    }

    #[test]
    fn test_vermin_growth_combined_storage_and_items() {
        let mut world = setup_world();

        // Add stored food
        world.resource_mut::<ColonyResources>().food = 1000.0;

        // Add item food
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Food,
                amount: 1000.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        // Should be roughly double the growth of just storage (assuming linear scaling)
        // 1000 stored -> 0.05 growth
        // 1000 item -> 0.05 growth (if implemented correctly)
        // Total -> 0.10
        assert!(
            vermin.severity > 0.06,
            "Combined sources should yield higher growth"
        );
    }
}
