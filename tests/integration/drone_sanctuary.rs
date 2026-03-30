#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::drone::Drone;
    use scale::layer1::execution::MovementTarget;
    use scale::layer1::hauling::haul_system;
    use scale::layer1::map::GridPosition;
    use scale::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::utility_ai::{ActionType, PopAction};
    use scale::layer1::zone::{ZoneGrid, ZoneType};

    #[test]
    fn test_drone_ignores_sanctuary_items() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Setup ZoneGrid with Sanctuary at (5,5)
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);

        // Spawn Drone with Haul action (simulating idle drone assigned work)
        let drone = world
            .spawn((
                Drone {
                    parent_hub: None,
                    is_active: true,
                },
                GridPosition { x: 0, y: 0 },
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn Item in Sanctuary
        let _item_sanctuary = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run Haul System
        haul_system(&mut world);

        // Assert Drone has NO target (ignores sanctuary item)
        assert!(
            world.get::<MovementTarget>(drone).is_none(),
            "Drone should ignore item in Sanctuary"
        );

        // Spawn Item OUTSIDE Sanctuary
        let item_outside = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 10.0,
                },
                GridPosition { x: 2, y: 2 },
            ))
            .id();

        // Run Haul System again
        haul_system(&mut world);

        // Assert Drone targets outside item
        let target = world.get::<MovementTarget>(drone);
        assert!(
            target.is_some(),
            "Drone should target item outside Sanctuary"
        );
        assert_eq!(target.unwrap().target_entity, item_outside);
    }
}
