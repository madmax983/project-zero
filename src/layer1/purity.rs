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
    #[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if let Some(&val) = self.overrides.get(&(x, y)) {
            return val;
        }

        // Simple pseudo-random hash
        let mut h = u64::from(self.seed);
        h = h.wrapping_add((i64::from(x) as u64).rotate_left(32));
        h = h.wrapping_add(i64::from(y) as u64);
        h = h.wrapping_mul(0x517c_c1b7_2722_0a95);
        h ^= h >> 32;

        // Normalize to 0.0 - 1.0
        // Use last 32 bits for better distribution if needed, but this is fine.
        (h as f32) / (u64::MAX as f32)
    }
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
        // Default purity should be reasonable (e.g., 1.0 or noise-based)
        // For testing, we might want a way to set it or rely on a deterministic seed.
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
}
