#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::atmosphere::{update_atmosphere_system, AtmosphereGrid};
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::integration::waste_pollution_bridge;
    use scale::layer1::map::GridPosition;
    use scale::layer1::resources::{ResourceItem, ResourceType};

    #[test]
    fn test_waste_item_emits_pollution() {
        let mut world = World::new();
        let grid = AtmosphereGrid::new(10, 10);
        world.insert_resource(grid);

        // Spawn Waste Item
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Waste,
                amount: 1.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run update (Bridge first, then Atmosphere update/diffusion)
        let mut schedule = Schedule::default();
        schedule.add_systems((
            waste_pollution_bridge,
            update_atmosphere_system.after(waste_pollution_bridge),
        ));
        schedule.run(&mut world);

        // Check pollution
        let grid = world.resource::<AtmosphereGrid>();
        assert!(
            grid.get(5, 5) > 0.0,
            "Waste item should emit pollution (got {})",
            grid.get(5, 5)
        );
    }

    #[test]
    fn test_landfill_emits_pollution() {
        let mut world = World::new();
        let grid = AtmosphereGrid::new(10, 10);
        world.insert_resource(grid);

        // Spawn Landfill
        world.spawn((
            Building {
                building_type: BuildingType::Landfill,
            },
            GridPosition { x: 2, y: 2 },
        ));

        // Run update
        let mut schedule = Schedule::default();
        schedule.add_systems((
            waste_pollution_bridge,
            update_atmosphere_system.after(waste_pollution_bridge),
        ));
        schedule.run(&mut world);

        // Check pollution
        let grid = world.resource::<AtmosphereGrid>();
        assert!(
            grid.get(2, 2) > 0.0,
            "Landfill should emit pollution (got {})",
            grid.get(2, 2)
        );
    }
}
