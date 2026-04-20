use bevy::prelude::*;

use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::physics::pressure::PressureGrid;

#[derive(Component)]
pub struct LostTech;

pub fn generate_tether_stump(mut terrain: ResMut<TerrainGrid>) {
    // Hardcoded center for minimal implementation
    let center_x = terrain.width / 2;
    let center_y = terrain.height / 2;

    // Create a 2x2 stump
    for x in center_x..center_x+2 {
        for y in center_y..center_y+2 {
            terrain.set(x, y, TerrainType::IndestructibleStump);
            // In the real TerrainGrid, we might need to set max_height per tile
            // terrain.set_max_height(x, y, u32::MAX); // Handled by TerrainGrid::get_max_build_height
        }
    }
}

pub fn initialize_atmosphere_gradient(pressure_grid: &mut PressureGrid) {
    for x in 0..pressure_grid.width() {
        for y in 0..pressure_grid.height() {
            for z in 0..pressure_grid.depth() {
                // Simple linear drop-off for minimal implementation
                let pressure = 1.0 - (z as f32 / 99.0).clamp(0.0, 1.0);
                pressure_grid.set_3d(x as i32, y as i32, z as i32, pressure);
            }
        }
    }
}

pub fn spawn_lost_tech_caches(mut commands: Commands, terrain: Res<TerrainGrid>) {
    let (stump_x, stump_y) = find_stump_center(&terrain).unwrap_or((50, 50));

    // Spawn one piece of tech high up
    commands.spawn((
        LostTech,
        Transform::from_xyz(stump_x as f32, stump_y as f32, 50.0),
    ));
}

// Helper for minimal implementation
fn find_stump_center(terrain: &TerrainGrid) -> Option<(usize, usize)> {
    for x in 0..terrain.width {
        for y in 0..terrain.height {
            if terrain.get(x, y) == Some(TerrainType::IndestructibleStump) {
                return Some((x, y));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::physics::pressure::PressureGrid;

    #[test]
    fn test_tether_stump_generation() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Assuming map generation plugin exists
        app.add_systems(Startup, generate_tether_stump);

        let terrain = crate::layer1::nature::terrain::generate_terrain(100, 100);
        app.insert_resource(terrain);

        // Act
        app.update();

        // Assert
        let terrain = app.world().resource::<TerrainGrid>();

        // Find the stump - should be a specific small footprint
        let mut stump_tiles = 0;
        for x in 0..100 {
            for y in 0..100 {
                if terrain.get(x, y) == Some(TerrainType::IndestructibleStump) {
                    stump_tiles += 1;
                }
            }
        }

        // Assert footprint is small (e.g., 2x2 or 3x3)
        assert!(stump_tiles > 0 && stump_tiles <= 9, "Stump should have a small footprint");
    }

    #[test]
    fn test_vertical_build_height_is_infinite_at_stump() {
        // Arrange
        let mut app = App::new();
        let terrain = crate::layer1::nature::terrain::generate_terrain(100, 100);
        app.insert_resource(terrain);
        app.add_systems(Startup, generate_tether_stump);
        app.update();

        // Act & Assert
        let terrain = app.world().resource::<TerrainGrid>();
        let (stump_x, stump_y) = find_stump_center(&terrain).unwrap();

        // Verify we can build at extreme heights at the stump coordinates
        assert_eq!(terrain.get_max_build_height(stump_x, stump_y), u32::MAX);
    }

    #[test]
    fn test_atmosphere_pressure_drops_with_height() {
        // Arrange
        // let mut app = App::new();
        let mut pressure_grid = PressureGrid::new_3d(100, 100, 100); // x, y, z

        // Act: Initialize atmosphere
        initialize_atmosphere_gradient(&mut pressure_grid);

        // Assert
        let ground_pressure = pressure_grid.get_3d(50, 50, 0);
        let high_altitude_pressure = pressure_grid.get_3d(50, 50, 50);

        assert!(high_altitude_pressure < ground_pressure, "Pressure should drop with altitude");
        assert!(high_altitude_pressure < 0.5, "High altitude should be dangerously thin"); // Assuming 1.0 is normal
    }

    #[test]
    fn test_lost_tech_spawns_at_high_altitude() {
        // Arrange
        let mut app = App::new();
        let terrain = crate::layer1::nature::terrain::generate_terrain(100, 100);
        app.insert_resource(terrain);
        app.add_systems(Startup, (generate_tether_stump, spawn_lost_tech_caches).chain());

        // Act
        app.update();

        // Assert
        let mut tech_query = app.world_mut().query::<(&Transform, &LostTech)>();
        let tech_count = tech_query.iter(app.world()).count();

        assert!(tech_count > 0, "Should spawn lost tech");

        for (transform, _) in tech_query.iter(app.world()) {
            assert!(transform.translation.z > 20.0, "Lost tech should only spawn at high altitude");
        }
    }
}
