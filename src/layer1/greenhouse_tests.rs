#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::farm::{Farm, produce_food_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::utility_types::{ActionType, PopAction};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_building_type_greenhouse_exists() {
        let b = BuildingType::Greenhouse;
        assert_eq!(b.label(), "Greenhouse");
        assert_eq!(b.char(), 'G');
    }

    #[test]
    fn test_greenhouse_ignores_winter_penalty() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        // Set Season to Winter (0.5 modifier usually)
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });

        // Spawn Greenhouse
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Greenhouse,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        // Run production
        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Normal Farm in Winter: 0.005 * 0.5 = 0.0025
        // Greenhouse in Winter: 0.005 * 1.0 = 0.005
        // Food starts at 10.0
        // Expected: 10.005
        assert!(
            (resources.food - 10.005).abs() < 0.0001,
            "Greenhouse should ignore winter penalty. Food: {}",
            resources.food
        );
    }

    #[test]
    fn test_greenhouse_cost() {
        // Should be expensive
        let cost = BuildingType::Greenhouse.cost(crate::layer1::building::MaterialType::Stone);
        assert!(cost.metal >= 10.0, "Greenhouse should require metal");
        assert!(
            cost.stone >= 20.0,
            "Greenhouse should require stone/glass equivalent"
        );
    }
}
