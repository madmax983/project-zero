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
}

impl DesignationType {
    /// Returns the character representation of the designation.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::designation::DesignationType;
    ///
    /// assert_eq!(DesignationType::Mine.char(), '⛏');
    /// ```
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Mine => '⛏',
            Self::Demolish => 'X',
        }
    }

    /// Returns a string slice representation of the designation.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::designation::DesignationType;
    ///
    /// assert_eq!(DesignationType::Mine.as_str(), "⛏");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mine => "⛏",
            Self::Demolish => "X",
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
#[derive(Resource, Default)]
pub struct DesignationMode {
    /// Whether designation mode is active.
    pub active: bool,
    /// The currently selected designation tool.
    pub tool: DesignationType,
    /// The cursor position for designation.
    pub cursor: GridPosition,
}

/// Checks if a designation can be placed at the given coordinates.
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
/// tiles[0] = TerrainType::Rock;
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(OccupiedTiles::default());
///
/// assert!(can_designate(&world, 0, 0, DesignationType::Mine));
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
    }
}

/// Attempts to apply a designation at the given coordinates.
/// Returns true if successful.
pub fn try_designate(world: &mut World, x: i32, y: i32, designation_type: DesignationType) -> bool {
    if !can_designate(world, x, y, designation_type) {
        return false;
    }

    world.spawn((Designation { designation_type }, GridPosition { x, y }));

    true
}

/// Attempts to remove any designation at the given coordinates.
/// Returns true if a designation was removed.
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
        assert_eq!(DesignationType::Mine.char(), '⛏');
        assert_eq!(DesignationType::Demolish.char(), 'X');
    }

    #[test]
    fn test_designation_type_as_str() {
        assert_eq!(DesignationType::Mine.as_str(), "⛏");
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
}
