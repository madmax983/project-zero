#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::beauty::{update_beauty_grid_system, BeautyGrid};
    use scale::layer1::clutter::ClutterGrid;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_clutter_reduces_beauty() {
        let mut world = scale::setup::setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles.fill(TerrainType::Grass);

        // Run beauty system before clutter
        let mut schedule = Schedule::default();
        schedule.add_systems(update_beauty_grid_system);
        schedule.run(&mut world);

        let initial_beauty = world.resource::<BeautyGrid>().get(5, 5);

        // Add clutter
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(5, 5, 50.0);

        // Run beauty system again
        schedule.run(&mut world);

        let new_beauty = world.resource::<BeautyGrid>().get(5, 5);
        assert!(
            new_beauty < initial_beauty,
            "Clutter should reduce beauty. Initial: {}, New: {}",
            initial_beauty,
            new_beauty
        );
    }

    #[test]
    fn test_clutter_increases_pathfinding_cost() {
        // Test that a path goes around a high-clutter area if possible
        let mut world = scale::setup::setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles.fill(TerrainType::Grass);

        // Path is from (1, 1) to (3, 1)
        // High clutter at (2, 1)
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(2, 1, 1000.0);

        let path = scale::layer1::pathfinding::find_path(&world, (1, 1), (3, 1));

        assert!(path.is_some(), "Path should be found");
        let path = path.unwrap();
        assert!(
            !path.contains(&(2, 1)),
            "Path should avoid high clutter area"
        );
    }
}
