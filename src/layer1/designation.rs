// src/layer1/designation.rs

use bevy_ecs::prelude::*;
use crate::layer1::{GridPosition, TerrainGrid, TerrainType};
use crate::layer1::OccupiedTiles;

/// Represents the type of operation designated for a tile.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DesignationType {
    /// Mark a rock tile for mining.
    #[default]
    Mine,
    /// Mark a structure or object for demolition.
    Demolish,
}

impl DesignationType {
    /// Returns the character used to represent this designation in the UI.
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Mine => '⛏',
            Self::Demolish => 'X',
        }
    }

    /// Returns the human-readable label for this designation.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Mine => "Mine",
            Self::Demolish => "Demolish",
        }
    }
}

/// Component attached to a tile entity indicating pending work.
#[derive(Component)]
pub struct Designation {
    /// The type of designation.
    pub designation_type: DesignationType,
}

/// Resource tracking the player's active designation tool.
#[derive(Resource, Default)]
pub struct DesignationMode {
    /// Whether designation mode is active.
    pub active: bool,
    /// The currently selected designation tool.
    pub tool: DesignationType,
    /// The current cursor position in designation mode.
    pub cursor: GridPosition,
}

/// Checks if a designation of the given type is valid at the specified coordinates.
#[must_use]
#[allow(clippy::cast_sign_loss)]
pub fn can_designate(
    world: &World,
    x: i32,
    y: i32,
    designation_type: DesignationType,
) -> bool {
    // Check bounds
    if x < 0 || y < 0 {
        return false;
    }

    // Check existing designation
    let existing = world
        .iter_entities()
        .filter_map(|e| {
            if e.contains::<Designation>() {
                e.get::<GridPosition>()
            } else {
                None
            }
        })
        .any(|pos| pos.x == x && pos.y == y);

    if existing {
        return false;
    }

    match designation_type {
        DesignationType::Mine => {
            let terrain = world.resource::<TerrainGrid>();
            terrain.get(x as usize, y as usize) == Some(TerrainType::Rock)
        }
        DesignationType::Demolish => {
            let occupied = world.resource::<OccupiedTiles>();
            // Only occupied tiles can be demolished
            occupied.0.contains(&(x, y))
        }
    }
}

/// Attempts to place a designation at the specified coordinates.
/// Returns true if successful.
pub fn try_designate(
    world: &mut World,
    x: i32,
    y: i32,
    designation_type: DesignationType,
) -> bool {
    if !can_designate(world, x, y, designation_type) {
        return false;
    }

    world.spawn((
        Designation { designation_type },
        GridPosition { x, y },
    ));

    true
}

/// Attempts to remove any designation at the specified coordinates.
/// Returns true if a designation was removed.
pub fn try_cancel_designation(world: &mut World, x: i32, y: i32) -> bool {
    let mut to_despawn = None;

    // Find designation at position
    for (entity, _, pos) in world.query::<(Entity, &Designation, &GridPosition)>().iter(world) {
        if pos.x == x && pos.y == y {
            to_despawn = Some(entity);
            break;
        }
    }

    to_despawn.is_some_and(|entity| {
        world.despawn(entity);
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::{GridPosition, OccupiedTiles};

    #[test]
    fn test_designation_type_variants() {
        let mine = DesignationType::Mine;
        let demolish = DesignationType::Demolish;
        assert_ne!(mine, demolish);
    }

    #[test]
    fn test_designation_type_char() {
        // Visualization is important for text UI, so we test the mapping exists
        assert_eq!(DesignationType::Mine.char(), '⛏');
        assert_eq!(DesignationType::Demolish.char(), 'X');
    }

    #[test]
    fn test_designation_component() {
        let designation = Designation {
            designation_type: DesignationType::Mine,
        };
        assert_eq!(designation.designation_type, DesignationType::Mine);
    }

    #[test]
    fn test_designation_mode_default() {
        let mode = DesignationMode::default();
        assert!(!mode.active);
        assert_eq!(mode.tool, DesignationType::Mine);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
    }

    #[test]
    fn test_designation_mode_toggle() {
        let mut mode = DesignationMode::default();
        mode.active = true;
        assert!(mode.active);
        mode.active = false;
        assert!(!mode.active);
    }

    #[test]
    fn test_can_designate_mine_valid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // Can mine Rock
        assert!(can_designate(&world, 5, 5, DesignationType::Mine));
    }

    #[test]
    fn test_can_designate_mine_invalid() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // Cannot mine Grass
        assert!(!can_designate(&world, 5, 5, DesignationType::Mine));
    }

    #[test]
    fn test_can_designate_demolish_valid() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        // Can demolish occupied tile
        assert!(can_designate(&world, 5, 5, DesignationType::Demolish));
    }

    #[test]
    fn test_can_designate_demolish_invalid() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Cannot demolish empty tile
        assert!(!can_designate(&world, 5, 5, DesignationType::Demolish));
    }

    #[test]
    fn test_try_designate_creates_entity() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let success = try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert!(success);

        let count = world.query::<(&Designation, &GridPosition)>().iter(&world).count();
        assert_eq!(count, 1);

        let (designation, pos) = world.query::<(&Designation, &GridPosition)>().single(&world);
        assert_eq!(designation.designation_type, DesignationType::Mine);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_try_designate_duplicates_ignored() {
        let mut world = World::new();
        // Setup valid rock
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // First designation
        try_designate(&mut world, 5, 5, DesignationType::Mine);

        // Second designation (same type/pos)
        let success = try_designate(&mut world, 5, 5, DesignationType::Mine);

        // Should return true (idempotent) or false?
        // Let's say false because "nothing happened"
        assert!(!success);

        // Count should still be 1
        let count = world.query::<&Designation>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_cancel_designation() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(OccupiedTiles::default());

        try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert_eq!(world.entities().len(), 1);

        let removed = try_cancel_designation(&mut world, 5, 5);
        assert!(removed);

        // Entity should be despawned or component removed
        // Since designation is the main component, entity despawn is cleaner
        assert_eq!(world.entities().len(), 0);
    }
}
