//! User intent and map designations.
//!
//! Designations are the primary way the player interacts with the simulation (The Mandate).
//! Instead of directly manipulating entities, the player "designates" a tile for an action
//! (e.g., "Mine here"), and the Pop AI (Souls) fulfills that request asynchronously.
//!
//! # Key Concepts
//!
//! * **Designation**: A persistent request attached to a specific map coordinate.
//! * **DesignationType**: The kind of request (Mine, Demolish).
//! * **Validation**: Rules for where designations can be placed (`can_designate`).

use crate::layer1::zone::ZoneType;
use crate::layer1::{GridPosition, OccupiedTiles, TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Types of designations a player can apply to the map.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DesignationType {
    /// Designate a rock tile for mining.
    #[default]
    Mine,
    /// Designate a building for demolition.
    Demolish,
    /// Designate a tree for chopping.
    Chop,
    /// Designate a building for repair.
    Repair,
    /// Set a zone type for a tile.
    SetZone(ZoneType),
    /// Designate an animal for taming.
    Tame,
    /// Designate flora for clearing.
    ClearFlora,
}

impl DesignationType {
    /// Returns the character representation of the designation.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::designation::DesignationType;
    ///
    /// assert_eq!(DesignationType::Mine.char(), '%');
    /// ```
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Mine => '%',
            Self::Demolish => 'X',
            Self::Chop => '/',
            Self::Repair => '+',
            Self::SetZone(_) => 'Z',
            Self::Tame => '♥',
            Self::ClearFlora => 'F',
        }
    }

    /// Returns a string slice representation of the designation.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::designation::DesignationType;
    ///
    /// assert_eq!(DesignationType::Mine.as_str(), "%");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mine => "%",
            Self::Demolish => "X",
            Self::Chop => "/",
            Self::Repair => "+",
            Self::SetZone(_) => "Z",
            Self::Tame => "♥",
            Self::ClearFlora => "F",
        }
    }

    /// Returns the human-readable label of the designation.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::designation::DesignationType;
    ///
    /// assert_eq!(DesignationType::Mine.label(), "Mine");
    /// ```
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Mine => "Mine",
            Self::Demolish => "Demolish",
            Self::Chop => "Chop",
            Self::Repair => "Repair",
            Self::SetZone(_) => "Set Zone",
            Self::Tame => "Tame",
            Self::ClearFlora => "Clear Flora",
        }
    }
}

/// Component attached to an entity representing a designation.
#[derive(Component)]
pub struct Designation {
    /// The type of designation.
    pub designation_type: DesignationType,
}

/// Resource tracking the player's current designation mode state.
///
/// This acts as the "Tool Controller" for the UI.
#[derive(Resource, Default)]
pub struct DesignationMode {
    /// Whether designation mode is active.
    pub active: bool,
    /// The currently selected designation tool.
    pub tool: DesignationType,
    /// The cursor position for designation.
    pub cursor: GridPosition,
    /// The starting corner of a drag rectangle (set on first press, cleared on second).
    pub drag_start: Option<GridPosition>,
}

/// Checks if a designation can be placed at the given coordinates.
///
/// This enforces game rules, such as "You can only mine Rock" or "You can only demolish buildings".
///
/// # Examples
///
/// ```
/// use scale::layer1::designation::{can_designate, DesignationType};
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use scale::layer1::building::OccupiedTiles;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let mut tiles = vec![TerrainType::Grass; 100];
/// tiles[0] = TerrainType::Rock; // (0,0) is Rock
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(OccupiedTiles::default());
///
/// // Can mine Rock
/// assert!(can_designate(&world, 0, 0, DesignationType::Mine));
/// // Cannot mine Grass (implied at 1,1)
/// assert!(!can_designate(&world, 1, 1, DesignationType::Mine));
/// ```
#[must_use]
#[allow(clippy::cast_sign_loss)]
pub fn can_designate(world: &World, x: i32, y: i32, designation_type: DesignationType) -> bool {
    // Check bounds (basic check, more detailed check in terrain/occupied logic)
    if x < 0 || y < 0 {
        return false;
    }

    // Check for existing designation at this location
    // Note: This linear query might be slow for many designations,
    // but acceptable for the MVP scope (designations < 1000).
    // Future optimization: Spatial index for designations.
    let existing = world
        .iter_entities()
        .filter_map(|e| {
            e.get::<GridPosition>()
                .and_then(|p| e.get::<Designation>().map(|_| p))
        })
        .any(|pos| pos.x == x && pos.y == y);

    if existing {
        return false;
    }

    match designation_type {
        DesignationType::Mine => {
            let terrain = world.resource::<TerrainGrid>();
            // Allow casting because we checked for negative above
            terrain.get(x as usize, y as usize) == Some(TerrainType::Rock)
        }
        DesignationType::Demolish => {
            let occupied = world.resource::<OccupiedTiles>();
            // Only occupied tiles can be demolished
            occupied.0.contains(&(x, y))
        }
        DesignationType::Chop => {
            let terrain = world.resource::<TerrainGrid>();
            // Allow casting because we checked for negative above
            terrain.get(x as usize, y as usize) == Some(TerrainType::Tree)
        }
        DesignationType::Repair => {
            let occupied = world.resource::<OccupiedTiles>();
            // Only occupied tiles can be repaired (assumes building)
            // Ideally check if building has Structure and < Max HP, but for MVP check occupancy is enough
            occupied.0.contains(&(x, y))
        }
        DesignationType::SetZone(_) => true,
        DesignationType::Tame => {
            // Must target a wild animal (Fauna without Tame component)
            // This is O(N) over all entities if we don't have spatial index, but okay for MVP
            world.iter_entities().any(|entity_ref| {
                if let Some(pos) = entity_ref.get::<GridPosition>()
                    && pos.x == x
                    && pos.y == y
                    && entity_ref.contains::<crate::layer1::fauna::Fauna>()
                {
                    return !entity_ref.contains::<crate::layer1::husbandry::Tame>();
                }
                false
            })
        }
        DesignationType::ClearFlora => {
            // Must target a tile with Flora
            world.iter_entities().any(|entity_ref| {
                if let Some(pos) = entity_ref.get::<GridPosition>() {
                    return pos.x == x
                        && pos.y == y
                        && entity_ref.contains::<crate::layer1::flora::Flora>();
                }
                false
            })
        }
    }
}

/// Attempts to apply a designation at the given coordinates.
///
/// Wrapper around `can_designate` that spawns the entity if valid.
///
/// # Returns
///
/// * `true` if the designation was successfully placed.
/// * `false` if the placement was invalid or a designation already exists.
///
/// # Examples
///
/// ```
/// use scale::layer1::designation::{try_designate, DesignationType};
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use scale::layer1::building::OccupiedTiles;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// // Setup valid condition (Rock)
/// let mut tiles = vec![TerrainType::Grass; 100];
/// tiles[55] = TerrainType::Rock; // (5,5)
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(OccupiedTiles::default());
///
/// assert!(try_designate(&mut world, 5, 5, DesignationType::Mine));
/// assert!(!try_designate(&mut world, 5, 5, DesignationType::Mine)); // Duplicate
/// ```
pub fn try_designate(world: &mut World, x: i32, y: i32, designation_type: DesignationType) -> bool {
    if !can_designate(world, x, y, designation_type) {
        return false;
    }

    world.spawn((Designation { designation_type }, GridPosition { x, y }));

    true
}

/// Designate all eligible tiles in a rectangle. Returns count of successful designations.
///
/// The rectangle is defined by two corners `(x1, y1)` and `(x2, y2)`. Corners can be
/// given in any order; the function normalizes to min/max internally.
///
/// # Examples
///
/// ```
/// use scale::layer1::designation::{try_designate_area, DesignationType};
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use scale::layer1::building::OccupiedTiles;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let mut tiles = vec![TerrainType::Grass; 100];
/// tiles[55] = TerrainType::Rock; // (5,5)
/// tiles[56] = TerrainType::Rock; // (6,5)
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(OccupiedTiles::default());
///
/// assert_eq!(try_designate_area(&mut world, 5, 5, 6, 5, DesignationType::Mine), 2);
/// ```
pub fn try_designate_area(
    world: &mut World,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    tool: DesignationType,
) -> u32 {
    let (min_x, max_x) = (x1.min(x2), x1.max(x2));
    let (min_y, max_y) = (y1.min(y2), y1.max(y2));
    let mut count = 0;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if try_designate(world, x, y, tool) {
                count += 1;
            }
        }
    }
    count
}

/// Attempts to remove any designation at the given coordinates.
///
/// Used when the user right-clicks or cancels a designation.
///
/// # Returns
///
/// * `true` if a designation was found and removed.
/// * `false` if no designation existed at that location.
pub fn try_cancel_designation(world: &mut World, x: i32, y: i32) -> bool {
    let mut to_despawn = None;

    // Find designation at position
    for (entity, _, pos) in world
        .query::<(Entity, &Designation, &GridPosition)>()
        .iter(world)
    {
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
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_designation_type_variants() {
        let mine = DesignationType::Mine;
        let demolish = DesignationType::Demolish;
        assert_ne!(mine, demolish);
    }

    #[test]
    fn test_designation_type_char() {
        // Visualization is important for text UI, so we test the mapping exists
        assert_eq!(DesignationType::Mine.char(), '%');
        assert_eq!(DesignationType::Demolish.char(), 'X');
    }

    #[test]
    fn test_designation_type_as_str() {
        assert_eq!(DesignationType::Mine.as_str(), "%");
        assert_eq!(DesignationType::Demolish.as_str(), "X");
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
        assert!(mode.drag_start.is_none());
    }

    #[test]
    fn test_designation_mode_toggle() {
        let mut mode = DesignationMode {
            active: true,
            ..Default::default()
        };
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

        let count = world
            .query::<(&Designation, &GridPosition)>()
            .iter(&world)
            .count();
        assert_eq!(count, 1);

        let (designation, pos) = world
            .query::<(&Designation, &GridPosition)>()
            .single(&world);
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
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert_eq!(world.entities().len(), 1);

        let removed = try_cancel_designation(&mut world, 5, 5);
        assert!(removed);

        // Entity should be despawned or component removed
        // Since designation is the main component, entity despawn is cleaner
        assert_eq!(world.entities().len(), 0);
    }

    #[test]
    fn test_designation_type_chop() {
        // Test new variant properties
        assert_eq!(DesignationType::Chop.char(), '/');
        assert_eq!(DesignationType::Chop.label(), "Chop");
    }

    #[test]
    fn test_can_designate_chop_valid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Should be able to chop a Tree
        assert!(can_designate(&world, 5, 5, DesignationType::Chop));
    }

    #[test]
    fn test_can_designate_chop_invalid() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Cannot chop Grass
        assert!(!can_designate(&world, 5, 5, DesignationType::Chop));
    }

    #[test]
    fn test_try_designate_area_single_tile() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5,5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let count = try_designate_area(&mut world, 5, 5, 5, 5, DesignationType::Mine);
        assert_eq!(count, 1);

        let designation_count = world.query::<&Designation>().iter(&world).count();
        assert_eq!(designation_count, 1);
    }

    #[test]
    fn test_try_designate_area_multi_tile_rectangle() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        // Fill a 3x2 rectangle with Rock at (2,3), (3,3), (4,3), (2,4), (3,4), (4,4)
        tiles[32] = TerrainType::Rock; // (2,3)
        tiles[33] = TerrainType::Rock; // (3,3)
        tiles[34] = TerrainType::Rock; // (4,3)
        tiles[42] = TerrainType::Rock; // (2,4)
        tiles[43] = TerrainType::Rock; // (3,4)
        tiles[44] = TerrainType::Rock; // (4,4)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // Drag from (4,4) to (2,3) — reversed corners
        let count = try_designate_area(&mut world, 4, 4, 2, 3, DesignationType::Mine);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_try_designate_area_mixed_valid_invalid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        // Only (5,5) and (6,5) are Rock in a 3-tile row
        tiles[55] = TerrainType::Rock; // (5,5)
        tiles[56] = TerrainType::Rock; // (6,5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // Area covers (5,5) to (7,5) — only 2 of 3 are valid
        let count = try_designate_area(&mut world, 5, 5, 7, 5, DesignationType::Mine);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_try_designate_area_no_valid_tiles() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100]; // All grass
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let count = try_designate_area(&mut world, 0, 0, 2, 2, DesignationType::Mine);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_set_zone_always_valid() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Can set zone anywhere
        assert!(can_designate(
            &world,
            0,
            0,
            DesignationType::SetZone(ZoneType::Bedroom)
        ));
    }
}

#[test]
fn test_can_designate_clear_flora() {
    let mut world = World::new();
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Grass; 100],
    });

    // Spawn Flora at (5, 5)
    world.spawn((
        crate::layer1::flora::Flora::default(),
        GridPosition { x: 5, y: 5 },
    ));

    // Can designate ClearFlora on Flora
    assert!(can_designate(&world, 5, 5, DesignationType::ClearFlora));

    // Cannot designate ClearFlora on empty tile
    assert!(!can_designate(&world, 5, 6, DesignationType::ClearFlora));
}
