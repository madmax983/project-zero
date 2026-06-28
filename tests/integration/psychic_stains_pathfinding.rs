#[cfg(test)]
mod tests {
    use scale::layer1::pathfinding::find_path;
    use scale::layer1::psychology::psychic_stains::PsychicStain;
    use scale::layer1::GridPosition;

    #[test]
    fn test_psychic_stain_increases_pathfinding_cost() {
        let mut world = scale::setup::setup_world();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

        let mut terrain = world.resource_mut::<scale::layer1::terrain::TerrainGrid>();
        terrain
            .tiles
            .fill(scale::layer1::terrain::TerrainType::Grass);

        // The direct path is (0,0) -> (1,0) -> (2,0) or similar.

        // We put heavy stains on the direct path.
        for x in 0..5 {
            world.spawn((
                GridPosition { x, y: 1 },
                PsychicStain {
                    trauma_level: 100.0,
                },
            ));
        }

        let start = (1, 0);
        let end = (1, 2);

        let path = find_path(&world, start, end);
        assert!(path.is_some());
        let path_unwrapped = path.unwrap();

        assert!(!path_unwrapped.contains(&(1, 1)));
    }
}
