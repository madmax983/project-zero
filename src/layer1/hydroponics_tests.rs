#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::farm::{produce_food_system, Farm};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::utility_types::{ActionType, PopAction};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<crate::layer1::eureka::EurekaEvent>>();
        world
    }

    #[test]
    fn test_hydroponics_consumes_water_and_power() {
        let mut world = setup_test_world();
        let res = ColonyResources { water: 100.0, food: 0.0, ..Default::default() };
        // power is not in ColonyResources directly, it's handled via PowerConsumer active state usually?
        // Or maybe ColonyResources has fuel?
        // Spec says "Consumes Water and Power".
        // 042 Energy says PowerConsumer consumes power from the grid.
        // If produce_food_system checks if it's powered, we need to simulate it being powered.
        // For this test, we might just set active=true manually if the system checks it.
        // But the system will likely check `PowerConsumer.active`.

        world.insert_resource(res);
        world.insert_resource(SeasonState {
            current_season: Season::Spring,
        }); // Default season

        // Spawn Hydroponics Bay
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::HydroponicsBay,
            },
            GridPosition { x: 5, y: 5 },
            // Manually add PowerConsumer for now (spawn_building does it in GREEN phase)
            // But if we use spawn_building helper, it might not add it yet.
            // Let's rely on manual component addition to test the SYSTEM logic.
            // Or better: The test in the spec uses `Building { ... }` and assumes components are there or system handles it.
            // If the system queries for PowerConsumer, we MUST add it here.
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        // Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(res.water < 100.0, "Hydroponics must consume water");

        // We can't easily check power consumption here because that's handled by power_grid_system/fuel system.
        // produce_food_system just checks if it IS powered.
        // The fact that it produced food (implied by water consumption) means it worked.
    }

    #[test]
    fn test_hydroponics_production_multiplier() {
        let mut world = setup_test_world();
        let res = ColonyResources { water: 100.0, food: 0.0, ..Default::default() };
        world.insert_resource(res);
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });

        // Hydroponics
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::HydroponicsBay,
            },
            GridPosition { x: 5, y: 5 },
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        // Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Base 0.005. Hydroponics multiplier 2.0. Winter ignore.
        // Expected ~0.01
        assert!(
            res.food >= 0.009,
            "Hydroponics should produce ~2x base yield"
        );
    }

    #[test]
    fn test_hydroponics_fails_without_water() {
        let mut world = setup_test_world();
        let res = ColonyResources { water: 0.0, food: 0.0, ..Default::default() };
        world.insert_resource(res);
        world.insert_resource(SeasonState {
            current_season: Season::Spring,
        });

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::HydroponicsBay,
            },
            GridPosition { x: 5, y: 5 },
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.food, 0.0, "Should not produce food without water");
    }

    #[test]
    fn test_hydroponics_fails_without_power() {
        let mut world = setup_test_world();
        let res = ColonyResources { water: 100.0, food: 0.0, ..Default::default() };
        world.insert_resource(res);
        world.insert_resource(SeasonState {
            current_season: Season::Spring,
        });

        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::HydroponicsBay,
            },
            GridPosition { x: 5, y: 5 },
            PowerConsumer {
                demand: 5.0,
                active: false,
            }, // Not powered
        ));

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.food, 0.0, "Should not produce food without power");
        // Also ensure water is NOT consumed
        assert_eq!(res.water, 100.0, "Should not consume water if not powered");
    }

    #[test]
    fn test_spawn_hydroponics_components() {
        let mut world = World::new();

        use crate::layer1::building::{try_place_building, OccupiedTiles};
        use crate::layer1::tech::{Tech, TechState};
        use crate::layer1::terrain::{TerrainGrid, TerrainType};

        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            metal: 100.0,
            stone: 100.0,
            ..Default::default()
        });
        let mut tech_state = TechState { total_capacity: 100.0, ..Default::default() };
        tech_state.unlock(Tech::Hydroponics);
        world.insert_resource(tech_state);
        world.insert_resource(crate::shared::log::MessageLog::default());

        let success = try_place_building(&mut world, 5, 5, BuildingType::HydroponicsBay);
        assert!(success, "Should place building");

        let entity = world
            .query_filtered::<Entity, With<Building>>()
            .single(&world);

        assert!(
            world.get::<Farm>(entity).is_some(),
            "Should have Farm component"
        );
        assert!(
            world.get::<PowerConsumer>(entity).is_some(),
            "Should have PowerConsumer component"
        );
    }
}
