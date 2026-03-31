#[cfg(test)]
mod tests {
    use crate::layer1::acoustic::{update_noise_system, NoiseMap};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pressure::PressureGrid;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_vacuum_blocks_noise() {
        let mut world = World::new();
        let size = 10;
        world.insert_resource(NoiseMap::new(size, size));

        let terrain = TerrainGrid {
            width: size,
            height: size,
            tiles: vec![TerrainType::Grass; size * size],
        };
        world.insert_resource(terrain);

        let mut atmos = PressureGrid::new(size, size);
        // Set (5,5) as noise source (Air)
        atmos.set(5, 5, 1.0);
        // Set (6,5) as Vacuum (0 pressure)
        atmos.set(6, 5, 0.0);
        // Set (7,5) as Listener (Air)
        atmos.set(7, 5, 1.0);

        world.insert_resource(atmos);

        // Spawn noise source
        world.spawn((
            GridPosition { x: 5, y: 5 },
            crate::layer1::acoustic::NoiseSource {
                intensity: 1.0,
                radius: 4.0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let noise = world.resource::<NoiseMap>();

        // Source should be loud
        assert!(noise.get(5, 5) > 0.9);
        // Vacuum should have 0 noise (physically impossible to carry sound, so it should just be ambient 0.1 or 0.0)
        assert!(noise.get(6, 5) <= 0.1);
        // Listener across the gap should hear NOTHING (ambient noise)
        assert!(noise.get(7, 5) <= 0.1);
    }

    #[test]
    fn test_sound_flanks_vacuum() {
        // Sound should travel around the vacuum gap if there is air
        let mut world = World::new();
        let size = 10;
        world.insert_resource(NoiseMap::new(size, size));

        let terrain = TerrainGrid {
            width: size,
            height: size,
            tiles: vec![TerrainType::Grass; size * size],
        };
        world.insert_resource(terrain);

        let mut atmos = PressureGrid::new(size, size);
        // 5,5 Source
        atmos.set(5, 5, 1.0);
        // 6,5 Vacuum wall
        atmos.set(6, 5, 0.0);
        // 7,5 Listener
        atmos.set(7, 5, 1.0);
        // 6,6 Air Bridge
        atmos.set(6, 6, 1.0);
        // 5,6 Air Bridge
        atmos.set(5, 6, 1.0);
        // 7,6 Air Bridge
        atmos.set(7, 6, 1.0);

        world.insert_resource(atmos);

        world.spawn((
            GridPosition { x: 5, y: 5 },
            crate::layer1::acoustic::NoiseSource {
                intensity: 1.0,
                radius: 4.0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let noise = world.resource::<NoiseMap>();
        // Should hear some noise via (5,5)->(5,6)->(6,6)->(7,6)->(7,5) or similar path
        assert!(noise.get(7, 5) > 0.1); // > ambient noise
    }
}
