#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::acoustic::{NoiseMap, NoiseSource, update_noise_system};
    use scale::layer1::building::{BuildingType, MaterialType, spawn_building_with_material};
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
    use scale::layer1::GridPosition;
    use scale::layer1::resources::ColonyResources;

    #[test]
    fn test_noisy_buildings_emit_noise() {
        let mut world = World::new();

        // Setup resources
        let width = 20;
        let height = 20;
        world.insert_resource(NoiseMap::new(width, height));
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::layer1::building::BuildMode::default());
        world.insert_resource(scale::shared::log::MessageLog::default());
        world.insert_resource(scale::layer1::tech::TechState::default());

        spawn_building_with_material(
            &mut world,
            10,
            10,
            BuildingType::LumberMill,
            MaterialType::Wood,
        );

        // Verify NoiseSource component exists
        let mut noise_source_query = world.query::<(&NoiseSource, &GridPosition)>();
        let count = noise_source_query.iter(&world).count();
        assert_eq!(count, 1, "LumberMill should have a NoiseSource component");

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        // Check NoiseMap
        let noise_map = world.resource::<NoiseMap>();
        let center_noise = noise_map.get(10, 10);

        // Ambient is 0.1. LumberMill should add significant noise.
        // Assuming intensity 0.5 or higher.
        assert!(center_noise > 0.2, "Noise level at LumberMill should be elevated. Got {}", center_noise);

        // Check falloff
        let distant_noise = noise_map.get(10, 15); // 5 tiles away
        assert!(distant_noise < center_noise, "Noise should fall off with distance");
        assert!(distant_noise >= 0.1, "Noise should be at least ambient");
    }

    #[test]
    fn test_quiet_buildings_do_not_emit_noise() {
        let mut world = World::new();

        let width = 20;
        let height = 20;
        world.insert_resource(NoiseMap::new(width, height));
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::layer1::tech::TechState::default());
        world.insert_resource(scale::shared::log::MessageLog::default());

        spawn_building_with_material(
            &mut world,
            5,
            5,
            BuildingType::Housing,
            MaterialType::Wood,
        );

        let mut noise_source_query = world.query::<(&NoiseSource, &GridPosition)>();
        let count = noise_source_query.iter(&world).count();
        assert_eq!(count, 0, "Housing should NOT have a NoiseSource component");

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let noise_map = world.resource::<NoiseMap>();
        let center_noise = noise_map.get(5, 5);

        // Should be ambient (0.1)
        assert!((center_noise - 0.1).abs() < f32::EPSILON, "Noise level at Housing should be ambient. Got {}", center_noise);
    }
}
