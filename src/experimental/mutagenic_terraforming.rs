//! Mutagenic Terraforming (Nova Feature).
//!
//! # The Spark
//! We have `WeatherType::MutagenicRain` and `TerrainGrid` (`TerrainType::Grass`, `TerrainType::Dirt`).
//!
//! # The Feature
//! When it rains `MutagenicRain`, exposed tiles (not under a roof) of `Grass` or `Dirt` have a small chance
//! to mutate into `TerrainType::SporeBloom`.
//!
//! # The Potential
//! Mutagenic rain isn't just an atmospheric hazard for Pops; it actively terraforms the colony,
//! replacing useful land with toxic fungal growth that must be dealt with, physically altering the landscape.

use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

pub fn mutagenic_terraforming_system(
    weather_state: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut terrain_grid: ResMut<TerrainGrid>,
) {
    if let Some(weather) = weather_state {
        if weather.current_weather == WeatherType::MutagenicRain {
            let mut rng = rand::thread_rng();
            let width = terrain_grid.width;
            let height = terrain_grid.height;

            for y in 0..height {
                for x in 0..width {
                    // Check if the tile is exposed (no roof)
                    let is_exposed = match &roof_grid {
                        Some(roofs) => !roofs.has_roof(x as i32, y as i32),
                        None => true,
                    };

                    if is_exposed {
                        if let Some(terrain) = terrain_grid.get(x, y) {
                            if matches!(terrain, TerrainType::Grass | TerrainType::Dirt) {
                                // 0.1% chance per tick per exposed tile
                                if rng.gen_bool(0.001) {
                                    terrain_grid.set(x, y, TerrainType::SporeBloom);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(mutagenic_terraforming_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_mutagenic_terraforming() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MutagenicRain,
            duration_remaining: 100,
        });

        let mut grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        // Set one to Dirt just to test both match arms
        grid.set(5, 5, TerrainType::Dirt);
        world.insert_resource(grid);

        // Run the system multiple times to statistically guarantee at least one mutation
        for _ in 0..10000 {
            world
                .run_system_once(mutagenic_terraforming_system)
                .unwrap();
        }

        let grid = world.resource::<TerrainGrid>();
        let mut mutated_count = 0;
        for y in 0..10 {
            for x in 0..10 {
                if let Some(TerrainType::SporeBloom) = grid.get(x, y) {
                    mutated_count += 1;
                }
            }
        }

        assert!(
            mutated_count > 0,
            "Mutagenic rain should terraform some tiles to SporeBloom"
        );
    }

    #[test]
    fn test_roofs_block_terraforming() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MutagenicRain,
            duration_remaining: 100,
        });

        let grid = TerrainGrid {
            width: 1,
            height: 1,
            tiles: vec![TerrainType::Grass; 1],
        };
        world.insert_resource(grid);

        let mut roofs = RoofGrid::new(1, 1);
        roofs.set(0, 0, true);
        world.insert_resource(roofs);

        // Run the system multiple times to statistically guarantee at least one mutation
        for _ in 0..10000 {
            world
                .run_system_once(mutagenic_terraforming_system)
                .unwrap();
        }

        let grid = world.resource::<TerrainGrid>();
        assert_eq!(
            grid.get(0, 0),
            Some(TerrainType::Grass),
            "Roofs should protect tiles from mutagenic terraforming"
        );
    }
}
