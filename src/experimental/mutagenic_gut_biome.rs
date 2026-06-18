//! Mutagenic Gut Biome (Nova Feature)
//!
//! # The Spark
//! The colony experiences toxic `MutagenicRain` that normally only affects the terrain
//! or causes surface-level mutations. Meanwhile, the `GutBiome` system tracks how familiar
//! pops are with different food types. What if the atmospheric mutagens are ingested?
//!
//! # The Feature
//! When `WeatherType::MutagenicRain` falls, any Pop caught outdoors (not under a roof)
//! will slowly have their gut bacteria mutated, passively increasing their familiarity
//! with `BiomeCategory::Xeno`.
//!
//! # The Potential
//! This connects environmental hazards to long-term dietary shifts. Players might intentionally
//! force their pops out into the mutagenic rain if they are low on standard crops and need
//! the colony to suddenly adapt to eating `AlienMeat` or `MysteryMeals` without sickness.

use crate::layer1::biology::gut_biome::{BiomeCategory, GutBiome};
use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use bevy_ecs::prelude::*;

/// System that mutates the gut biome of outdoors pops during mutagenic rain.
pub fn mutagenic_gut_biome_system(
    weather: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut pops: Query<(&mut GutBiome, &GridPosition), With<Pop>>,
) {
    if let Some(w) = weather {
        if w.current_weather == WeatherType::MutagenicRain {
            let grid = roof_grid.as_deref();

            for (mut gut_biome, pos) in pops.iter_mut() {
                // If there's no roof grid, we assume outdoors.
                // If there is, we check if the pop is under a roof (indoors).
                let indoors = grid.is_some_and(|g| {
                    if pos.x < 0 || pos.y < 0 {
                        false
                    } else {
                        let x = pos.x as usize;
                        let y = pos.y as usize;
                        if x < g.width && y < g.height {
                            g.has_roof[y * g.width + x]
                        } else {
                            false
                        }
                    }
                });

                if !indoors {
                    gut_biome.adapt(BiomeCategory::Xeno);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_mutagenic_gut_biome_outdoors() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MutagenicRain,
            duration_remaining: 100,
        });

        // Add pop outdoors (no RoofGrid)
        let mut initial_biome = GutBiome::default();
        // Start with 0 xeno familiarity
        initial_biome.set_familiarity(BiomeCategory::Xeno, 0.0);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, initial_biome))
            .id();

        world.run_system_once(mutagenic_gut_biome_system).unwrap();

        let biome = world.get::<GutBiome>(pop).unwrap();
        assert!(biome.get_familiarity(BiomeCategory::Xeno) > 0.0, "Gut biome should adapt to Xeno during mutagenic rain outdoors");
    }

    #[test]
    fn test_mutagenic_gut_biome_indoors() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MutagenicRain,
            duration_remaining: 100,
        });

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.has_roof[5 * 10 + 5] = true; // Pop is indoors
        world.insert_resource(roof_grid);

        let mut initial_biome = GutBiome::default();
        initial_biome.set_familiarity(BiomeCategory::Xeno, 0.0);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, initial_biome))
            .id();

        world.run_system_once(mutagenic_gut_biome_system).unwrap();

        let biome = world.get::<GutBiome>(pop).unwrap();
        assert_eq!(biome.get_familiarity(BiomeCategory::Xeno), 0.0, "Gut biome should NOT adapt to Xeno if indoors");
    }

    #[test]
    fn test_mutagenic_gut_biome_wrong_weather() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain, // Normal rain
            duration_remaining: 100,
        });

        let mut initial_biome = GutBiome::default();
        initial_biome.set_familiarity(BiomeCategory::Xeno, 0.0);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, initial_biome))
            .id();

        world.run_system_once(mutagenic_gut_biome_system).unwrap();

        let biome = world.get::<GutBiome>(pop).unwrap();
        assert_eq!(biome.get_familiarity(BiomeCategory::Xeno), 0.0, "Gut biome should only mutate in MutagenicRain");
    }
}
