use crate::layer1::beauty::BeautyGrid;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;
use image::{ImageBuffer, Rgb};

#[derive(Resource)]
pub struct CartographyExportConfig {
    pub export_path: String,
    pub trigger_export: bool,
}

impl Default for CartographyExportConfig {
    fn default() -> Self {
        Self {
            export_path: "colony_map.png".to_string(),
            trigger_export: false,
        }
    }
}

pub fn map_export_system(
    config_opt: Option<ResMut<CartographyExportConfig>>,
    terrain: Res<TerrainGrid>,
    beauty_opt: Option<Res<BeautyGrid>>,
) {
    let mut config = match config_opt {
        Some(c) => c,
        None => return,
    };
    if !config.trigger_export {
        return;
    }

    let width = terrain.width as u32;
    let height = terrain.height as u32;
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let tile = terrain
                .get(x as usize, y as usize)
                .unwrap_or(TerrainType::Grass);
            let mut color: [u8; 3] = match tile {
                TerrainType::Grass => [34, 139, 34],      // Forest Green
                TerrainType::Dirt => [139, 69, 19],       // Saddle Brown
                TerrainType::Rock => [128, 128, 128],     // Gray
                TerrainType::Water => [30, 144, 255],     // Dodger Blue
                TerrainType::Tree => [0, 100, 0],         // Dark Green
                TerrainType::Path => [210, 180, 140],     // Tan
                TerrainType::Shrub => [154, 205, 50],     // Yellow Green
                TerrainType::Sapling => [107, 142, 35],   // Olive Drab
                TerrainType::DeepRock => [105, 105, 105], // Dim Gray
                TerrainType::MagmaRock => [178, 34, 34],  // Firebrick
                TerrainType::SporeBloom => [148, 0, 211], // Dark Violet
                TerrainType::Artifact => [255, 215, 0],   // Gold
            };

            // Optionally blend with beauty
            if let Some(ref beauty) = beauty_opt {
                let b_val = beauty.get(x as usize, y as usize);
                if b_val > 5.0 {
                    color[0] = color[0].saturating_add(30);
                    color[1] = color[1].saturating_add(30);
                    color[2] = color[2].saturating_add(30);
                } else if b_val < -5.0 {
                    color[0] = color[0].saturating_sub(30);
                    color[1] = color[1].saturating_sub(30);
                    color[2] = color[2].saturating_sub(30);
                }
            }

            img.put_pixel(x, y, Rgb(color));
        }
    }

    if let Err(e) = img.save(&config.export_path) {
        log::error!("Failed to export map to {}: {}", config.export_path, e);
    } else {
        log::info!("Successfully exported map to {}", config.export_path);
    }

    config.trigger_export = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_export_system() {
        let mut world = World::new();
        world.insert_resource(CartographyExportConfig {
            export_path: "test_export.png".to_string(),
            trigger_export: true,
        });

        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(map_export_system);
        schedule.run(&mut world);

        let config = world.resource::<CartographyExportConfig>();
        assert!(!config.trigger_export); // Should be reset after run
        assert!(std::path::Path::new("test_export.png").exists());

        // Clean up
        std::fs::remove_file("test_export.png").unwrap_or_default();
    }
}
