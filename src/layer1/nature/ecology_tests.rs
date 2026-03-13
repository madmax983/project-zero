#[cfg(test)]
mod tests {
    use crate::layer1::ecology::{process_ecological_succession, EcologyConfig};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_terrain_type_shrub() {
        assert_eq!(TerrainType::Shrub.name(), "Shrub");
        assert!(TerrainType::Shrub.is_walkable());
    }

    #[test]
    fn test_terrain_type_sapling() {
        assert_eq!(TerrainType::Sapling.name(), "Sapling");
        assert!(TerrainType::Sapling.is_walkable());
    }

    #[test]
    fn test_dirt_to_grass_pioneer() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Dirt; 9];
        // Center is dirt
        world.insert_resource(TerrainGrid {
            width: 3,
            height: 3,
            tiles,
        });

        let config = EcologyConfig { growth_rate: 1.0, pioneer_chance: 1.0, ..Default::default() };
        world.insert_resource(config);

        // Run succession
        process_ecological_succession(&mut world);

        let grid = world.resource::<TerrainGrid>();
        // Center should eventually become Grass (pioneer species)
        assert!(grid.tiles.contains(&TerrainType::Grass));
    }

    #[test]
    fn test_grass_to_sapling_seeding() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 9];
        tiles[0] = TerrainType::Tree; // (0,0) is a seed source
        world.insert_resource(TerrainGrid {
            width: 3,
            height: 3,
            tiles,
        });

        let config = EcologyConfig { growth_rate: 1.0, seed_spread_chance: 1.0, ..Default::default() };
        // Make sure pioneer chance (random shrub) doesn't interfere if we are testing seeding specifically?
        // Actually, logic usually prioritizes seeding or checks it.
        // We want to test Sapling appearance.
        world.insert_resource(config);

        // Run succession multiple times to ensure coverage (avoid flaky test due to random sampling)
        for _ in 0..10 {
            process_ecological_succession(&mut world);
        }

        let grid = world.resource::<TerrainGrid>();
        // Neighbors of tree should have chance to become Sapling
        // Specifically check (0,1) or (1,0) or (1,1)
        let sapling_count = grid
            .tiles
            .iter()
            .filter(|&&t| t == TerrainType::Sapling)
            .count();
        assert!(sapling_count > 0);
    }

    #[test]
    fn test_sapling_growth_to_tree() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Sapling; 9];
        world.insert_resource(TerrainGrid {
            width: 3,
            height: 3,
            tiles,
        });

        let config = EcologyConfig { growth_rate: 1.0, maturation_chance: 1.0, ..Default::default() };
        world.insert_resource(config);

        // Run succession multiple times to simulate time
        for _ in 0..10 {
            process_ecological_succession(&mut world);
        }

        let grid = world.resource::<TerrainGrid>();
        let tree_count = grid
            .tiles
            .iter()
            .filter(|&&t| t == TerrainType::Tree)
            .count();
        assert!(tree_count > 0);
    }

    #[test]
    fn test_rock_does_not_grow() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Rock; 9];
        world.insert_resource(TerrainGrid {
            width: 3,
            height: 3,
            tiles,
        });
        world.insert_resource(EcologyConfig::default());

        process_ecological_succession(&mut world);

        let grid = world.resource::<TerrainGrid>();
        assert!(grid.tiles.iter().all(|&t| t == TerrainType::Rock));
    }
}
