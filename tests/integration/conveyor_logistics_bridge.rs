#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::logistics::conveyor::{
        conveyor_system, hopper_system, inserter_system, BeltVariant, ConveyorBelt,
        Hopper, Inserter,
    };
    use scale::layer1::core::map::GridPosition;
    use scale::layer1::{TerrainGrid, TerrainType};
    use scale::layer1::energy::PowerConsumer;
    use scale::layer1::ResourceItem;
    use scale::layer1::economy::resources::{ColonyResources, ResourceType};
    use scale::layer1::architecture::building::{Building, BuildingType, Direction};

    #[test]
    fn test_conveyor_logistics_end_to_end() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Required resources
        app.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Let's set stone to 0 to make test logic simple
        let mut res = ColonyResources::default();
        res.max_stone = 100.0;
        res.stone = 0.0;
        app.insert_resource(res);

        // Schedule systems in order
        app.add_systems(
            Update,
            (inserter_system, conveyor_system, hopper_system).chain(),
        );

        // 1. Inserter at (1, 0)
        // Picks up from West (0, 0), drops off to East (2, 0)
        app.world_mut().spawn((
            Inserter {
                pickup_direction: Direction::West,
                dropoff_direction: Direction::East,
            },
            GridPosition { x: 1, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));

        // 2. Conveyor at (2, 0)
        // Moves East to (3, 0)
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::East,
                speed: 1.0,
                variant: BeltVariant::Standard,
            },
            GridPosition { x: 2, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));

        // 3. Hopper at (3, 0)
        // Consumes items
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Hopper,
            },
            Hopper,
            GridPosition { x: 3, y: 0 },
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        // 4. Item at (0, 0)
        let item = app
            .world_mut()
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 5.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        app.update();

        assert!(app.world().get_entity(item).is_err(), "Item should be consumed by hopper");

        let res = app.world().resource::<ColonyResources>();
        assert!((res.stone - 5.0).abs() < f32::EPSILON, "Hopper should add 5.0 stone to ColonyResources, but has {}", res.stone);
    }
}
