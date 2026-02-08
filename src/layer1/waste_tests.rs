#[cfg(test)]
mod tests {
    use crate::layer1::GridPosition;
    use crate::layer1::beauty::{BeautyGrid, update_beauty_grid_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::process_refining_system;
    use crate::layer1::resources::{ColonyResources, RefiningProgress, ResourceItem, ResourceType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    // 1. ResourceType::Waste exists
    #[test]
    fn test_resource_type_waste() {
        let waste = ResourceType::Waste;
        // Verify it can be used in ResourceItem
        let item = ResourceItem {
            resource_type: waste,
            amount: 1.0,
        };
        assert_eq!(item.resource_type, ResourceType::Waste);
    }

    // 2. Refining produces Waste
    #[test]
    fn test_refining_produces_waste_item() {
        let mut world = World::new();
        // Setup Resources (Wood -> Planks + Waste)
        // We need enough wood for multiple attempts
        let resources = ColonyResources {
            wood: 200.0,
            planks: 0.0,
            max_wood: 1000.0,
            max_planks: 1000.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Spawn Multiple LumberMills to ensure probability hits
        // 50% chance per operation. With 100 mills, chance of failure is negligible.
        for i in 0..100 {
            world.spawn((
                Building {
                    building_type: BuildingType::LumberMill,
                },
                GridPosition { x: i, y: 0 },
                RefiningProgress {
                    current: 9.9,
                    max: 10.0,
                }, // Almost done
            ));
            world.spawn((Pop, GridPosition { x: i, y: 1 }));
        }

        // Run system
        process_refining_system(&mut world);

        // Check for Waste item on ground
        let mut found_waste = false;
        let mut query = world.query::<(&GridPosition, &ResourceItem)>();
        for (_, item) in query.iter(&world) {
            if item.resource_type == ResourceType::Waste {
                found_waste = true;
                break;
            }
        }
        assert!(
            found_waste,
            "Refining should spawn Waste item at building location (probabilistic)"
        );
    }

    // 3. Waste item emits negative beauty
    #[test]
    fn test_waste_item_negative_beauty() {
        let mut world = World::new();
        world.insert_resource(BeautyGrid::new(10, 10));

        // Spawn Waste Item
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Waste,
                amount: 1.0,
            },
            GridPosition { x: 2, y: 2 },
        ));

        // Run beauty system
        let _ = world.run_system_once(update_beauty_grid_system);

        let grid = world.resource::<BeautyGrid>();
        // Expect negative value (e.g., -5.0)
        assert!(
            grid.get(2, 2) < 0.0,
            "Waste item should emit negative beauty"
        );
    }

    // 4. Landfill Building
    #[test]
    fn test_landfill_building_properties() {
        let lf = BuildingType::Landfill;
        // Should have negative intrinsic beauty
        assert!(lf.beauty_value() < 0.0);
    }

    // 5. Landfill increases Waste Cap
    #[test]
    fn test_landfill_increases_waste_cap() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Landfill
        world.spawn((
            Building {
                building_type: BuildingType::Landfill,
            },
            // Stockpile component with waste_bonus
            crate::layer1::stockpile::Stockpile {
                waste_bonus: 100.0,
                ..Default::default()
            },
        ));

        // Run cap update system
        let _ = world.run_system_once(crate::layer1::stockpile::update_resource_caps_system);

        let res = world.resource::<ColonyResources>();
        assert!(res.max_waste >= 100.0);
    }
}
