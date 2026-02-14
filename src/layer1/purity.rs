#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::suboptimal_flops
)]

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// A map storing the purity of terrain at given coordinates.
#[derive(Resource, Default)]
pub struct PurityMap {
    /// Seed for deterministic generation.
    seed: u32,
    /// Override values for testing.
    overrides: HashMap<(i32, i32), f32>,
}

impl PurityMap {
    /// Creates a new `PurityMap` with the given seed.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            overrides: HashMap::new(),
        }
    }

    /// Sets an override purity value for a specific coordinate.
    pub fn set_override(&mut self, x: i32, y: i32, value: f32) {
        self.overrides.insert((x, y), value);
    }

    /// Gets the purity at the given coordinates (0.0 to 1.0).
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if let Some(&val) = self.overrides.get(&(x, y)) {
            return val;
        }

        // Use value noise with frequency scaling to create veins
        // Frequency 0.1 gives features roughly 10 tiles wide
        self.value_noise(x as f32 * 0.1, y as f32 * 0.1)
    }

    /// Generates 2D value noise at the given float coordinates.
    fn value_noise(&self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;

        let tx = x - x.floor();
        let ty = y - y.floor();

        // Smooth interpolation (quintic curve: 6t^5 - 15t^4 + 10t^3)
        // Smoother than cubic (3t^2 - 2t^3)
        let sx = tx * tx * tx * (tx * (tx * 6.0 - 15.0) + 10.0);
        let sy = ty * ty * ty * (ty * (ty * 6.0 - 15.0) + 10.0);

        // Hash values at 4 corners
        let c00 = self.hash(xi, yi);
        let c10 = self.hash(xi + 1, yi);
        let c01 = self.hash(xi, yi + 1);
        let c11 = self.hash(xi + 1, yi + 1);

        // Bilinear interpolation
        let nx0 = lerp(c00, c10, sx);
        let nx1 = lerp(c01, c11, sx);

        lerp(nx0, nx1, sy)
    }

    /// Deterministic hash function returning 0.0 - 1.0
    fn hash(&self, x: i32, y: i32) -> f32 {
        let mut h = u64::from(self.seed);
        h = h.wrapping_add((i64::from(x) as u64).rotate_left(32));
        h = h.wrapping_add(i64::from(y) as u64);
        h = h.wrapping_mul(0x517c_c1b7_2722_0a95);
        h ^= h >> 32;
        (h as f32) / (u64::MAX as f32)
    }
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::resources::{
        ColonyResources, MiningProgress, ResourceItem, ResourceType, mine_rock,
    };
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_purity_map_resource_exists() {
        let _map = PurityMap::default();
    }

    #[test]
    fn test_mine_rock_high_purity_yields_ore() {
        let mut world = World::new();
        // Setup world with 1 Rock
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[0] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10)); // Dependency
        world.insert_resource(crate::shared::log::MessageLog::default()); // Added for logging

        // Setup PurityMap with High Purity (1.0) at (0,0)
        let mut purity_map = PurityMap::default();
        purity_map.set_override(0, 0, 1.0); // Helper for testing
        world.insert_resource(purity_map);

        // Spawn Designation
        let designation = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                MiningProgress {
                    current: 9.0,
                    max: 10.0,
                },
            ))
            .id();

        // Mine
        mine_rock(&mut world, designation, 1.0);

        // Assert: Should contain Ore
        let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        let has_ore = items.iter().any(|i| i.resource_type == ResourceType::Ore);
        assert!(has_ore, "High purity should yield Ore");

        // Assert: Should NOT contain Waste (at 1.0 purity)
        let has_waste = items.iter().any(|i| i.resource_type == ResourceType::Waste);
        assert!(!has_waste, "High purity should not yield Waste");
    }

    #[test]
    fn test_mine_rock_low_purity_yields_waste() {
        let mut world = World::new();
        // Setup Rock
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[0] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Setup PurityMap with Low Purity (0.0)
        let mut purity_map = PurityMap::default();
        purity_map.set_override(0, 0, 0.0);
        world.insert_resource(purity_map);

        let designation = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                MiningProgress {
                    current: 9.0,
                    max: 10.0,
                },
            ))
            .id();

        mine_rock(&mut world, designation, 1.0);

        // Assert: Should contain Waste
        let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        let has_waste = items.iter().any(|i| i.resource_type == ResourceType::Waste);
        assert!(has_waste, "Low purity should yield Waste");

        // Assert: Should NOT contain Ore (at 0.0 purity)
        let has_ore = items.iter().any(|i| i.resource_type == ResourceType::Ore);
        assert!(!has_ore, "Low purity should not yield Ore");
    }

    #[test]
    fn test_get_purity_consistent() {
        let map = PurityMap::new(12345); // Seed
        let p1 = map.get(10, 10);
        let p2 = map.get(10, 10);
        assert!(
            (p1 - p2).abs() < f32::EPSILON,
            "Purity should be deterministic for same seed/coord"
        );
    }

    #[test]
    fn test_purity_map_smoothness() {
        let map = PurityMap::new(42);
        let mut total_delta = 0.0;
        let count = 100;

        // Sample horizontal deltas
        for x in 0..count {
            let p1 = map.get(x, 0);
            let p2 = map.get(x + 1, 0);
            total_delta += (p1 - p2).abs();
        }

        let avg_delta = total_delta / count as f32;

        // With frequency 0.1, we expect change to be relatively small per step.
        // It shouldn't be tiny (like 0.001) but definitely less than white noise (0.33).
        println!("Avg Delta: {}", avg_delta);
        assert!(
            avg_delta < 0.15,
            "Purity map should be smooth (veins), but avg delta was {}",
            avg_delta
        );
    }
}
