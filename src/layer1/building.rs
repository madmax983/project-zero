// src/layer1/building.rs

use super::GridPosition;
use super::farm::Farm;
use super::housing::Housing;
use super::social::Tavern;
use super::stockpile::Stockpile;
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use crate::layer1::tech::{Library, Tech, TechState};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Building types available for construction.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, EnumIter)]
pub enum BuildingType {
    /// Basic shelter for pops.
    #[default]
    Housing,
    /// Agricultural building for food production.
    Farm,
    /// Storage for resources.
    Stockpile,
    /// Refines Wood into Planks.
    LumberMill,
    /// Refines Stone into Blocks.
    StoneMason,
    /// Refines Ore into Metal.
    Smelter,
    /// Refines Metal and Wood into Tools.
    Smithy,
    /// Social gathering place.
    Tavern,
    /// Research center for Knowledge.
    Library,
}

impl BuildingType {
    /// Returns the tech required to build this building, if any.
    #[must_use]
    pub const fn required_tech(&self) -> Option<Tech> {
        match self {
            Self::Smelter | Self::Smithy => Some(Tech::MetalWorking),
            Self::Tavern => Some(Tech::SocialStructures),
            _ => None,
        }
    }

    /// Returns the human-readable label of the building.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::building::BuildingType;
    ///
    /// assert_eq!(BuildingType::Housing.label(), "Housing");
    /// ```
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Housing => "Housing",
            Self::Farm => "Farm",
            Self::Stockpile => "Stockpile",
            Self::LumberMill => "Lumber Mill",
            Self::StoneMason => "Stone Mason",
            Self::Smelter => "Smelter",
            Self::Smithy => "Smithy",
            Self::Tavern => "Tavern",
            Self::Library => "Library",
        }
    }

    /// Returns the character representation of the building.
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Housing => 'H',
            Self::Farm => 'F',
            Self::Stockpile => '=',
            Self::LumberMill => 'L',
            Self::StoneMason => 'M',
            Self::Smelter => 'S',
            Self::Smithy | Self::Tavern => 'T',
            Self::Library => '?', // Placeholder
        }
    }

    /// Returns the resource cost to build this building.
    #[must_use]
    pub fn cost(&self) -> ColonyResources {
        match self {
            Self::Housing => ColonyResources {
                wood: 10.0,
                ..Default::default()
            },
            Self::Farm => ColonyResources {
                wood: 20.0,
                stone: 5.0,
                ..Default::default()
            },
            Self::Stockpile => ColonyResources {
                wood: 50.0,
                ..Default::default()
            },
            Self::LumberMill | Self::Smithy => ColonyResources {
                wood: 30.0,
                stone: 10.0,
                ..Default::default()
            },
            Self::StoneMason => ColonyResources {
                wood: 40.0,
                stone: 20.0,
                ..Default::default()
            },
            Self::Smelter => ColonyResources {
                wood: 20.0,
                stone: 50.0,
                ..Default::default()
            },
            Self::Tavern => ColonyResources {
                wood: 40.0,
                stone: 10.0,
                ..Default::default()
            },
            Self::Library => ColonyResources::default(),
        }
    }

    /// Returns the next building type in the cycle.
    ///
    /// # Panics
    ///
    /// Panics if the `BuildingType` has no variants (which should never happen).
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::building::BuildingType;
    ///
    /// assert_eq!(BuildingType::Housing.next(), BuildingType::Farm);
    /// ```
    #[must_use]
    pub fn next(&self) -> Self {
        let mut iter = Self::iter();
        while let Some(current) = iter.next() {
            if &current == self {
                return iter.next().unwrap_or_else(|| Self::iter().next().unwrap());
            }
        }
        Self::default()
    }
}

/// Building component - attached to building entities.
#[derive(Component)]
pub struct Building {
    /// The type of this building.
    pub building_type: BuildingType,
}

/// Build mode state resource.
#[derive(Resource, Default)]
pub struct BuildMode {
    /// Whether build mode is currently active.
    pub active: bool,
    /// The current cursor position in grid coordinates.
    pub cursor: GridPosition,
    /// The currently selected building type.
    pub selected: BuildingType,
}

/// Tracks which tiles have buildings (for placement validation).
#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(i32, i32)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlacementError {
    OutOfBounds,
    Occupied,
    InvalidTerrain(TerrainType),
}

fn validate_building_placement(world: &World, x: i32, y: i32) -> Result<(), PlacementError> {
    let terrain = world.resource::<TerrainGrid>();
    let occupied = world.resource::<OccupiedTiles>();

    // Check bounds
    if x < 0 || y < 0 {
        return Err(PlacementError::OutOfBounds);
    }

    // Check terrain
    #[allow(clippy::cast_sign_loss)]
    let tile = terrain
        .get(x as usize, y as usize)
        .ok_or(PlacementError::OutOfBounds)?;

    match tile {
        TerrainType::Water | TerrainType::Rock => Err(PlacementError::InvalidTerrain(tile)),
        _ => {
            // Check occupation
            if occupied.0.contains(&(x, y)) {
                Err(PlacementError::Occupied)
            } else {
                Ok(())
            }
        }
    }
}

/// Check if a building can be placed at the given position.
#[must_use]
pub fn can_place_building(world: &World, x: i32, y: i32) -> bool {
    validate_building_placement(world, x, y).is_ok()
}

fn handle_placement_error(world: &mut World, error: PlacementError) {
    let reason = match error {
        PlacementError::OutOfBounds => "Out of bounds",
        PlacementError::Occupied => "Location occupied",
        PlacementError::InvalidTerrain(TerrainType::Water) => "Cannot build on Water",
        PlacementError::InvalidTerrain(TerrainType::Rock) => "Cannot build on Rock",
        PlacementError::InvalidTerrain(_) => "Cannot build here",
    };

    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add(format!("Failed: {reason}"));
    }
}

fn spawn_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) {
    let mut entity = world.spawn((Building { building_type }, GridPosition { x, y }));

    match building_type {
        BuildingType::Housing => {
            entity.insert(Housing::default());
        }
        BuildingType::Farm => {
            entity.insert(Farm::default());
        }
        BuildingType::Stockpile => {
            entity.insert(Stockpile::default());
        }
        BuildingType::LumberMill
        | BuildingType::StoneMason
        | BuildingType::Smelter
        | BuildingType::Smithy => {
            entity.insert(RefiningProgress {
                current: 0.0,
                max: 10.0,
            });
        }
        BuildingType::Tavern => {
            entity.insert(Tavern::default());
        }
        BuildingType::Library => {
            entity.insert(Library);
        }
    }
}

/// Attempt to place a building at the given position.
/// Returns true if successful, false if placement blocked.
///
/// This will:
/// 1. Check `can_place_building` (bounds, terrain, occupation).
/// 2. Spawn a building entity with the correct components (e.g., `Housing` or `Farm`).
/// 3. Mark the tile as occupied in `OccupiedTiles`.
///
/// # Examples
///
/// ```
/// use scale::layer1::building::{try_place_building, BuildingType, OccupiedTiles};
/// use scale::layer1::resources::ColonyResources;
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let tiles = vec![TerrainType::Grass; 100]; // 10x10 grass
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(OccupiedTiles::default());
/// world.insert_resource(ColonyResources {
///     wood: 100.0,
///     ..Default::default()
/// });
///
/// let placed = try_place_building(&mut world, 5, 5, BuildingType::Housing);
/// assert!(placed);
/// ```
pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
    if let Err(e) = validate_building_placement(world, x, y) {
        handle_placement_error(world, e);
        return false;
    }

    // Check Tech requirements
    if let Some(tech) = building_type.required_tech() {
        // We use get_resource because TechState might not be initialized in some tests
        // (though we should initialize it)
        // If it's missing, we default to "locked" to be safe.
        let tech_unlocked = world
            .get_resource::<TechState>()
            .is_some_and(|state| state.is_unlocked(tech));

        if !tech_unlocked {
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add(format!("Requires technology: {}", tech.label()));
            }
            return false;
        }
    }

    // Check affordability
    let cost = building_type.cost();
    let can_afford = {
        let resources = world.resource::<ColonyResources>();
        resources.can_afford(&cost)
    };

    if !can_afford {
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add(format!(
                "Not enough resources for {}",
                building_type.label()
            ));
        }
        return false;
    }

    // Deduct cost
    world.resource_mut::<ColonyResources>().deduct(&cost);

    // Spawn building
    spawn_building(world, x, y, building_type);

    // Mark tile occupied
    world.resource_mut::<OccupiedTiles>().0.insert((x, y));

    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add(format!("Construction started: {}", building_type.label()));
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_building_type_default() {
        let bt = BuildingType::default();
        assert_eq!(bt, BuildingType::Housing);
    }

    #[test]
    fn test_building_type_labels() {
        assert_eq!(BuildingType::Housing.label(), "Housing");
        assert_eq!(BuildingType::Farm.label(), "Farm");
    }

    #[test]
    fn test_building_type_next() {
        assert_eq!(BuildingType::Housing.next(), BuildingType::Farm);
        assert_eq!(BuildingType::Farm.next(), BuildingType::Stockpile);
        assert_eq!(BuildingType::Stockpile.next(), BuildingType::LumberMill);
        assert_eq!(BuildingType::LumberMill.next(), BuildingType::StoneMason);
        assert_eq!(BuildingType::StoneMason.next(), BuildingType::Smelter);
        assert_eq!(BuildingType::Smelter.next(), BuildingType::Smithy);
        assert_eq!(BuildingType::Smithy.next(), BuildingType::Tavern);
        assert_eq!(BuildingType::Tavern.next(), BuildingType::Library);
        assert_eq!(BuildingType::Library.next(), BuildingType::Housing);
    }

    #[test]
    fn test_building_component_creation() {
        let building = Building {
            building_type: BuildingType::Farm,
        };
        assert_eq!(building.building_type, BuildingType::Farm);
    }

    #[test]
    fn test_build_mode_default() {
        let mode = BuildMode::default();
        assert!(!mode.active);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
        assert_eq!(mode.selected, BuildingType::Housing);
    }

    #[test]
    fn test_build_mode_toggle() {
        let mut mode = BuildMode::default();
        assert!(!mode.active);

        mode.active = true;
        assert!(mode.active);

        mode.active = !mode.active;
        assert!(!mode.active);
    }

    #[test]
    fn test_build_mode_cursor_movement() {
        let mut mode = BuildMode::default();
        mode.cursor.x = 5;
        mode.cursor.y = 10;

        mode.cursor.x += 1;
        mode.cursor.y -= 1;

        assert_eq!(mode.cursor.x, 6);
        assert_eq!(mode.cursor.y, 9);
    }

    #[test]
    fn test_build_mode_type_cycling() {
        let mut mode = BuildMode::default();
        assert_eq!(mode.selected, BuildingType::Housing);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Farm);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Stockpile);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::LumberMill);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::StoneMason);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Smelter);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Smithy);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Tavern);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Library);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Housing);
    }

    #[test]
    fn test_occupied_tiles_default() {
        let occupied = OccupiedTiles::default();
        assert!(occupied.0.is_empty());
    }

    #[test]
    fn test_occupied_tiles_insertion() {
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 10));
        assert!(occupied.0.contains(&(5, 10)));
        assert!(!occupied.0.contains(&(5, 11)));
    }

    #[test]
    fn test_can_place_on_grass() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(can_place, "Should be able to place on grass");
    }

    #[test]
    fn test_cannot_place_on_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on water");
    }

    #[test]
    fn test_cannot_place_on_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on rock");
    }

    #[test]
    fn test_cannot_place_on_occupied() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on occupied tile");
    }

    #[test]
    fn test_cannot_place_out_of_bounds() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        assert!(!can_place_building(&world, -1, 5), "Negative x");
        assert!(!can_place_building(&world, 5, -1), "Negative y");
        assert!(!can_place_building(&world, 10, 5), "X out of bounds");
        assert!(!can_place_building(&world, 5, 10), "Y out of bounds");
    }

    #[test]
    fn test_place_building_success() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 1, "Should have spawned one building");

        let occupied = world.resource::<OccupiedTiles>();
        assert!(
            occupied.0.contains(&(5, 5)),
            "Tile should be marked occupied"
        );
    }

    #[test]
    fn test_place_building_failure_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 0, "Should not spawn building on water");
    }

    #[test]
    fn test_building_has_position() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 7, 3, BuildingType::Housing);

        let (pos, _) = world.query::<(&GridPosition, &Building)>().single(&world);
        assert_eq!(pos.x, 7);
        assert_eq!(pos.y, 3);
    }

    #[test]
    fn test_place_farm_adds_farm_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let farm_count = world.query::<&Farm>().iter(&world).count();
        assert_eq!(farm_count, 1, "Should have added Farm component");
    }

    #[test]
    fn test_place_building_log_messages() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water; // (5,5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        // Test Water failure
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);
        assert!(!success);
        let log = world.resource::<MessageLog>();
        assert_eq!(
            log.messages.back().unwrap().text,
            "Failed: Cannot build on Water"
        );

        // Test OutOfBounds failure
        let success = try_place_building(&mut world, -1, 5, BuildingType::Housing);
        assert!(!success);
        let log = world.resource::<MessageLog>();
        assert_eq!(log.messages.back().unwrap().text, "Failed: Out of bounds");

        // Test Success
        let success = try_place_building(&mut world, 0, 0, BuildingType::Housing);
        assert!(success);
        let log = world.resource::<MessageLog>();
        assert_eq!(
            log.messages.back().unwrap().text,
            "Construction started: Housing"
        );
    }

    #[test]
    fn test_housing_cost() {
        let cost = BuildingType::Housing.cost();
        assert!((cost.wood - 10.0).abs() < f32::EPSILON);
        assert!((cost.stone - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_farm_cost() {
        let cost = BuildingType::Farm.cost();
        assert!((cost.wood - 20.0).abs() < f32::EPSILON);
        assert!((cost.stone - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_can_afford_success() {
        let cost = ColonyResources {
            wood: 10.0,
            stone: 0.0,
            ..Default::default()
        };
        let available = ColonyResources {
            wood: 15.0,
            stone: 5.0,
            ..Default::default()
        };

        assert!(available.can_afford(&cost));
    }

    #[test]
    fn test_can_afford_failure() {
        let cost = ColonyResources {
            wood: 10.0,
            stone: 0.0,
            ..Default::default()
        };
        let available = ColonyResources {
            wood: 5.0,
            stone: 5.0,
            ..Default::default()
        };

        assert!(!available.can_afford(&cost));
    }

    #[test]
    fn test_try_place_building_deducts_resources() {
        let mut world = World::new();
        // Setup terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Setup resources (enough for Housing: 10 wood)
        world.insert_resource(ColonyResources {
            wood: 15.0,
            ..Default::default()
        });

        // Attempt placement
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(success);

        // Verify deduction
        let resources = world.resource::<ColonyResources>();
        assert!((resources.wood - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_try_place_building_fails_insufficient_funds() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Setup resources (not enough for Housing)
        world.insert_resource(ColonyResources {
            wood: 5.0,
            ..Default::default()
        });

        // Attempt placement
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(!success);

        // Verify no deduction
        let resources = world.resource::<ColonyResources>();
        assert!((resources.wood - 5.0).abs() < f32::EPSILON);

        // Verify no building
        assert!(world.query::<&Building>().iter(&world).count() == 0);
    }

    #[test]
    fn test_place_housing_adds_housing_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Housing);

        let housing_count = world.query::<&Housing>().iter(&world).count();
        assert_eq!(housing_count, 1, "Should have added Housing component");
    }

    #[test]
    fn test_place_stockpile_adds_stockpile_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0, // Stockpile needs 50 wood
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Stockpile);

        let stockpile_count = world.query::<&Stockpile>().iter(&world).count();
        assert_eq!(stockpile_count, 1, "Should have added Stockpile component");
    }

    #[test]
    fn test_build_on_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        // Building on tree should be allowed
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);
        assert!(success, "Should be able to build on Tree");

        // Verify terrain is STILL Tree (current behavior)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Tree));

        // Verify building exists
        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 1);
    }
}
