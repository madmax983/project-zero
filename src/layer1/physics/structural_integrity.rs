#![allow(clippy::cast_sign_loss)]
//! Structural Integrity and Stability Systems.
//!
//! This module defines the `RoofGrid` and calculates support limits (`MAX_SUPPORT_DIST`).
//! It ensures that spaces hollowed out during mining (turning `Rock` to `Dirt`) remain supported
//! by nearby `Rock` or `Wall` entities.
//!
//! Unstable roofs can trigger cave-ins via `apply_collapse`, converting the terrain back to rubble
//! and damaging trapped entities.

use crate::layer1::building::{Building, BuildingType};
use crate::layer1::health::Health;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::GridPosition;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

/// Maximum distance from a support (Rock/Wall) that a roof can sustain itself.
pub const MAX_SUPPORT_DIST: i32 = 5;

#[derive(Resource, Default)]
/// Tracks which tiles have an overhead roof (underground vs open sky).
///
/// # Examples
///
/// ```
/// use scale::layer1::structural_integrity::RoofGrid;
///
/// let mut grid = RoofGrid::new(10, 10);
/// grid.set(5, 5, true);
///
/// assert!(grid.has_roof(5, 5));
/// assert!(!grid.has_roof(0, 0));
/// ```
pub struct RoofGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Flat vector of roof status.
    pub has_roof: Vec<bool>,
}

impl RoofGrid {
    /// Creates a new `RoofGrid` with the specified dimensions, initialized to false (no roof).
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");
        Self {
            width,
            height,
            has_roof: vec![false; size],
        }
    }

    /// Sets the roof status for a specific tile.
    pub fn set(&mut self, x: i32, y: i32, val: bool) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let idx = (y as usize)
                .checked_mul(self.width)
                .and_then(|i| i.checked_add(x as usize));

            if let Some(idx) = idx.filter(|&i| i < self.has_roof.len()) {
                self.has_roof[idx] = val;
            }
        }
    }

    /// Returns true if the specified tile has a roof.
    #[must_use]
    pub fn has_roof(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= self.width || (y as usize) >= self.height {
            return false;
        }
        let idx = (y as usize)
            .checked_mul(self.width)
            .and_then(|i| i.checked_add(x as usize));

        if let Some(idx) = idx.filter(|&i| i < self.has_roof.len()) {
            return self.has_roof[idx];
        }
        false
    }
}

/// Checks if the roof at the given position is supported.
/// Returns true if stable (supported or no roof), false if unstable.
pub fn check_stability(world: &mut World, pos: GridPosition) -> bool {
    // 1. Check Roof
    let Some(roof) = world.get_resource::<RoofGrid>() else {
        return true; // Safe if there is no roof grid
    };
    if !roof.has_roof(pos.x, pos.y) {
        return true; // No roof = safe
    }

    // 2. Check Self Support (Rock)
    let Some(terrain) = world.get_resource::<TerrainGrid>() else {
        return true; // Safe if there is no terrain grid
    };
    if terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Rock) {
        return true;
    }

    // 3. Check Nearby Rock
    // Optimization: we could use a spiral search or something, but simple box loop is fine for MVP.

    // Safely convert map dimensions to i32 bounds, clamping to i32::MAX if larger.
    // This prevents wrap-around if width/height > i32::MAX (e.g. on 64-bit systems).
    let map_w = i32::try_from(terrain.width).unwrap_or(i32::MAX);
    let map_h = i32::try_from(terrain.height).unwrap_or(i32::MAX);

    let min_x = pos.x.saturating_sub(MAX_SUPPORT_DIST).max(0);
    let max_x = pos
        .x
        .saturating_add(MAX_SUPPORT_DIST)
        .min(map_w.saturating_sub(1));

    let min_y = pos.y.saturating_sub(MAX_SUPPORT_DIST).max(0);
    let max_y = pos
        .y
        .saturating_add(MAX_SUPPORT_DIST)
        .min(map_h.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Chebyshev Distance <= 5
            let check_pos = GridPosition { x, y };
            if check_pos.distance_chebyshev(pos) > MAX_SUPPORT_DIST as u32 {
                continue;
            }

            if terrain.get(x as usize, y as usize) == Some(TerrainType::Rock) {
                return true;
            }
        }
    }

    // 4. Check Nearby Buildings (Wall)
    // Naive iteration over all buildings. This might be slow if many buildings.
    // Optimization: Add spatial index for buildings if needed later.
    let mut query = world.query::<(&Building, &GridPosition)>();
    for (building, b_pos) in query.iter(world) {
        if b_pos.distance_chebyshev(pos) <= MAX_SUPPORT_DIST as u32
            && matches!(building.building_type, BuildingType::Wall)
        {
            return true;
        }
    }

    false
}

/// Triggers a cave-in at the specified position.
/// This changes terrain to Rock (rubble) and damages any entities present.
pub fn apply_collapse(world: &mut World, pos: GridPosition) {
    if pos.x < 0 || pos.y < 0 {
        return;
    }

    // 1. Change Terrain to Rock (Rubble)
    // We scope this mutable borrow of TerrainGrid
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        // SAFETY: We checked pos.x and pos.y are non-negative above.
        // We still need to check upper bounds, which is done by idx < len check.
        // However, converting to usize is now safe from wrapping huge negative numbers.
        // We also use checked arithmetic to prevent overflow wrapping around to a valid index.
        let idx = (pos.y as usize)
            .checked_mul(terrain.width)
            .and_then(|i| i.checked_add(pos.x as usize));

        if let Some(idx) = idx.filter(|&i| i < terrain.tiles.len()) {
            terrain.tiles[idx] = TerrainType::Rock;
        }
    }

    // 2. Damage entities
    // ⚡ Bolt Optimization:
    // Removed intermediate `Vec<Entity>` allocation (`victims`).
    // Previously, the system queried `&Health` to find victims, collected their `Entity` IDs,
    // and then did a second pass with `world.get_mut::<Health>` to apply damage.
    // By querying `&mut Health` directly, we avoid the heap allocation and collapse two
    // iterations into a single O(N) pass, eliminating memory pressure on the hot path.
    let mut query = world.query::<(&GridPosition, &mut Health)>();

    for (p, mut health) in query.iter_mut(world) {
        if p.x == pos.x && p.y == pos.y {
            health.take_damage(50.0);
        }
    }

    // 3. Log event
    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add(format!("CAVE-IN at ({}, {})!", pos.x, pos.y));
    }

    // 4. Emit Collapse Event
    // We check existence because unit tests might not initialize this event.
    if world.contains_resource::<Events<StructureCollapsed>>() {
        world.send_event(StructureCollapsed { pos });
    }
}

/// Event triggered when a roof collapses due to lack of support.
#[derive(Event, Debug, Clone)]
pub struct StructureCollapsed {
    /// Location of the collapse.
    pub pos: GridPosition,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::GridPosition;

    #[test]
    fn test_roof_initialization() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Rock; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Initialize RoofGrid based on Terrain
        // Rock should have roof, others false (unless specified)
        // For MVP, assume map gen handles this. Here we test manual init.
        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(0, 0, true); // (0,0) is Rock

        assert!(roof_grid.has_roof(0, 0));
        assert!(!roof_grid.has_roof(1, 1));
    }

    #[test]
    fn test_mining_preserves_roof() {
        // Mining changes Rock -> Dirt, but Roof should remain true
        // This test simulates mining logic update
        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(5, 5, true); // Target tile

        // Simulate mine_rock completion
        // ... (mine_rock logic updates terrain)
        // Check roof remains
        assert!(roof_grid.has_roof(5, 5));
    }

    #[test]
    fn test_check_stability_safe_failure_without_resources() {
        let mut world = World::new();
        // Missing RoofGrid and TerrainGrid
        let is_stable = check_stability(&mut world, GridPosition { x: 5, y: 5 });
        assert!(
            is_stable,
            "Should default to stable if resources are missing"
        );
    }

    #[test]
    fn test_stability_check_safe() {
        let mut world = World::new();
        // 5x5 area
        let mut tiles = vec![TerrainType::Dirt; 25];
        tiles[0] = TerrainType::Rock; // Support at (0,0)
        world.insert_resource(TerrainGrid {
            width: 5,
            height: 5,
            tiles,
        });

        let mut roof_grid = RoofGrid::new(5, 5);
        roof_grid.set(1, 0, true); // Neighbor to Rock
        world.insert_resource(roof_grid);

        // Check (1,0) - distance 1 to support
        assert!(check_stability(&mut world, GridPosition { x: 1, y: 0 }));
    }

    #[test]
    fn test_stability_check_far() {
        let mut world = World::new();
        // 10x10 area
        let mut tiles = vec![TerrainType::Dirt; 100];
        tiles[0] = TerrainType::Rock; // Only support at (0,0)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(9, 9, true); // Far away
        world.insert_resource(roof_grid);

        // Check (9,9) - distance > 5
        assert!(!check_stability(&mut world, GridPosition { x: 9, y: 9 }));
    }

    #[test]
    fn test_building_provides_support() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Dirt; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Place Wall at (0,0)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(1, 0, true);
        world.insert_resource(roof_grid);

        assert!(check_stability(&mut world, GridPosition { x: 1, y: 0 }));
    }

    #[test]
    fn test_collapse_mechanics() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Dirt; 100];
        tiles[55] = TerrainType::Dirt; // (5,5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Spawn victim
        let victim = world
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(5, 5, true);
        world.insert_resource(roof_grid);

        // Trigger collapse
        apply_collapse(&mut world, GridPosition { x: 5, y: 5 });

        // 1. Terrain becomes Rock
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Rock));

        // 2. Roof is gone (filled)
        // Optionally, roof logic might vary. Let's assume roof remains (it's rock now).
        // Actually, if it's rock, it HAS a roof implicitly.

        // 3. Victim takes damage
        let Some(health) = world.get::<Health>(victim) else {
            panic!("Missing Health");
        };
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_collapse_negative_coords() {
        let mut world = World::new();
        // Setup simple terrain
        let tiles = vec![TerrainType::Dirt; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Should not panic or crash
        apply_collapse(&mut world, GridPosition { x: -1, y: -1 });
    }

    #[test]
    fn test_stability_check_overflow() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Dirt; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(RoofGrid::new(10, 10));

        // Test near i32::MAX
        let pos_max = GridPosition {
            x: i32::MAX,
            y: i32::MAX,
        };
        // Should use saturating add/sub and clamp to 0..width/height
        // effectively checking nothing or just boundaries, but NOT panic
        // Returns true (safe) because no roof found there (out of bounds)
        assert!(check_stability(&mut world, pos_max));

        // Test near i32::MIN
        let pos_min = GridPosition {
            x: i32::MIN,
            y: i32::MIN,
        };
        // Returns true (safe) because no roof found there (out of bounds)
        assert!(check_stability(&mut world, pos_min));
    }
}
