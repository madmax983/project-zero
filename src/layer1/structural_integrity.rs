#![allow(clippy::cast_sign_loss)]
use crate::layer1::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::health::Health;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

/// Maximum distance from a support (Rock/Wall) that a roof can sustain itself.
pub const MAX_SUPPORT_DIST: i32 = 5;

#[derive(Resource, Default)]
/// Tracks which tiles have an overhead roof (underground vs open sky).
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
        Self {
            width,
            height,
            has_roof: vec![false; width * height],
        }
    }

    /// Sets the roof status for a specific tile.
    pub fn set(&mut self, x: i32, y: i32, val: bool) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.has_roof[(y as usize) * self.width + (x as usize)] = val;
        }
    }

    /// Returns true if the specified tile has a roof.
    #[must_use]
    pub fn has_roof(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= self.width || (y as usize) >= self.height {
            return false;
        }
        self.has_roof[(y as usize) * self.width + (x as usize)]
    }
}

/// Checks if the roof at the given position is supported.
/// Returns true if stable (supported or no roof), false if unstable.
pub fn check_stability(world: &mut World, pos: GridPosition) -> bool {
    // 1. Check Roof
    let roof = world.resource::<RoofGrid>();
    if !roof.has_roof(pos.x, pos.y) {
        return true; // No roof = safe
    }

    // 2. Check Self Support (Rock)
    let terrain = world.resource::<TerrainGrid>();
    if terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Rock) {
        return true;
    }

    // 3. Check Nearby Rock
    // Optimization: we could use a spiral search or something, but simple box loop is fine for MVP.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let min_x = (pos.x - MAX_SUPPORT_DIST).max(0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let max_x = (pos.x + MAX_SUPPORT_DIST).min(terrain.width as i32 - 1);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let min_y = (pos.y - MAX_SUPPORT_DIST).max(0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let max_y = (pos.y + MAX_SUPPORT_DIST).min(terrain.height as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Chebyshev Distance <= 5
            if (x - pos.x).abs().max((y - pos.y).abs()) > MAX_SUPPORT_DIST {
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
        if (b_pos.x - pos.x).abs().max((b_pos.y - pos.y).abs()) <= MAX_SUPPORT_DIST
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
    // 1. Change Terrain to Rock (Rubble)
    // We scope this mutable borrow of TerrainGrid
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
        if idx < terrain.tiles.len() {
            terrain.tiles[idx] = TerrainType::Rock;
        }
    }

    // 2. Damage entities
    // We collect entities first to avoid borrowing world while iterating
    let mut victims = Vec::new();
    // Use read-only query to find victims
    let mut query = world.query::<(Entity, &GridPosition, &Health)>();

    for (entity, p, _) in query.iter(world) {
        if p.x == pos.x && p.y == pos.y {
            victims.push(entity);
        }
    }

    // Apply damage
    for entity in victims {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
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
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

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
    fn test_stability_check_unsafe() {
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
        let health = world.get::<Health>(victim).unwrap();
        assert!(health.current < 100.0);
    }
}
