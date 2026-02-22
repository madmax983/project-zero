#[cfg(test)]
mod tests {
    use crate::layer1::building::BuildingType;
    use crate::layer1::farm::consume_food_system;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::get_refining_recipe;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::spoilage::spoilage_system;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_rations_resource_defaults() {
        let resources = ColonyResources::default();
        assert_eq!(resources.rations, 0.0);
        assert!(resources.max_rations > 0.0);
    }

    #[test]
    fn test_smokehouse_recipe() {
        let resources = ColonyResources {
            food: 5.0,
            wood: 1.0,
            rations: 0.0,
            max_rations: 10.0,
            ..ColonyResources::default()
        };

        // 5 Food + 1 Wood -> 5 Rations
        let (can_refine, input, output, _) =
            get_refining_recipe(BuildingType::Smokehouse, &resources);

        assert!(can_refine);
        assert_eq!(input.food, 5.0);
        assert_eq!(input.wood, 1.0);
        assert_eq!(output.rations, 5.0);
    }

    #[test]
    fn test_rations_decay_slower_than_food() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 1000.0;
        resources.rations = 1000.0;
        world.insert_resource(resources);
        // VerminState is optional in spoilage_system, we don't insert it.

        // Need to run system once. Bevy requires the system to be run via schedule or run_system_once.
        // We use run_system_once.
        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, spoilage_system).unwrap();

        let res = world.resource::<ColonyResources>();
        let food_loss = 1000.0 - res.food;
        let ration_loss = 1000.0 - res.rations;

        // Rations should decay at 10% the rate of Food
        assert!(ration_loss < food_loss);
        // Using epsilon for float comparison.
        // Expected decay: food * GLOBAL, rations * GLOBAL * 0.1
        // So ration_loss should be approx food_loss * 0.1
        assert!((ration_loss - (food_loss * 0.1)).abs() < 0.001);
    }

    #[test]
    fn test_consume_priority_eats_fresh_food_first() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0, // Just enough for one meal
            rations: 10.0,
            ..Default::default()
        });

        // Spawn hungry pop
        world.spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 1.0,
                ..Default::default()
            }, // Very hungry
        ));

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, consume_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Should eat the fresh food first
        assert!(res.food < 1.0);
        assert_eq!(res.rations, 10.0);
    }

    #[test]
    fn test_consume_eats_rations_if_no_fresh_food() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 0.0,
            rations: 10.0,
            ..Default::default()
        });

        // Spawn hungry pop
        world.spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 1.0,
                ..Default::default()
            },
        ));

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, consume_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Should eat rations
        assert!(res.rations < 10.0);
    }
}
