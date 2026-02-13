// src/layer1/building.rs

//! Building placement and types.
//!
//! Buildings are the primary structures in the colony. They provide housing,
//! production, defense, and social functions.
//!
//! # Core Systems
//!
//! *   **Placement:** Buildings are placed on the [`TerrainGrid`] using [`try_place_building`].
//! *   **Cost:** Each [`BuildingType`] has a [`ColonyResources`] cost (see [`BuildingType::cost`]).
//! *   **Tech:** Some buildings require specific [`Tech`] to be unlocked (see [`BuildingType::required_tech`]).
//! *   **Obstacles:** Most buildings block movement, but some (like Farms/Stockpiles) are walkable.
//!
//! # Entities
//!
//! A built structure is an entity with:
//! *   [`Building`]: The marker component containing the [`BuildingType`].
//! *   [`GridPosition`]: Its location on the map.
//! *   [`crate::layer1::structure::Structure`]: Health and durability.
//! *   Specific Logic Components: e.g., [`Housing`], [`Farm`], [`Stockpile`].

use super::GridPosition;
use super::beauty::BeautySource;
use super::farm::Farm;
use super::fire::Flammable;
use super::housing::Housing;
use super::social::Tavern;
use super::stockpile::Stockpile;
use crate::layer1::energy::{Conduit, PowerConsumer, PowerSource};
use crate::layer1::heirloom::AncientStructure;
use crate::layer1::lighting::LightSource;
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use crate::layer1::tech::{Library, Tech, TechState};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::trade::TradeDepot;
use crate::layer1::water::{MAX_HYDRATION, WaterSource};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Material types for buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum MaterialType {
    /// Basic wood material (Flammable).
    #[default]
    Wood,
    /// Durable stone material.
    Stone,
    /// Strong metal material.
    Metal,
    /// Luxurious gold material (High Beauty).
    Gold,
}

impl MaterialType {
    /// Returns true if the material is flammable.
    #[must_use]
    pub const fn flammability(&self) -> bool {
        matches!(self, Self::Wood)
    }

    /// Returns the HP modifier for this material.
    #[must_use]
    pub const fn hp_modifier(&self) -> f32 {
        match self {
            Self::Wood => 1.0,
            Self::Stone => 4.0,
            Self::Metal => 3.0,
            Self::Gold => 0.5,
        }
    }

    /// Returns the beauty modifier for this material.
    #[must_use]
    pub const fn beauty_modifier(&self) -> f32 {
        match self {
            Self::Wood | Self::Metal => 0.0,
            Self::Stone => 1.0,
            Self::Gold => 10.0,
        }
    }

    /// Returns the label of the material.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Wood => "Wood",
            Self::Stone => "Stone",
            Self::Metal => "Metal",
            Self::Gold => "Gold",
        }
    }

    /// Returns the next material in the cycle.
    ///
    /// # Panics
    /// Panics if enum has no variants.
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

/// Component defining the material of a building.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Material(pub MaterialType);

/// Building types available for construction.
///
/// This enum defines all constructible structures in the game. It contains metadata
/// for costs, tech requirements, appearance, and placement rules.
///
/// # Examples
///
/// Checking costs and labels:
///
/// ```
/// use scale::layer1::building::{BuildingType, MaterialType};
///
/// let housing = BuildingType::Housing;
/// assert_eq!(housing.label(), "Housing");
///
/// let cost = housing.cost(MaterialType::default());
/// assert_eq!(cost.wood, 10.0);
/// assert_eq!(cost.stone, 0.0);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, EnumIter)]
pub enum BuildingType {
    /// Basic shelter for pops.
    #[default]
    Housing,
    /// Agricultural building for food production.
    Farm,
    /// Source of water hydration.
    Well,
    /// Storage for resources.
    Stockpile,
    /// Refines Food into Rations.
    Smokehouse,
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
    /// Farming building for fiber production.
    Plantation,
    /// Refines Fiber into Cloth.
    Weaver,
    /// Refines Cloth into Clothing.
    Tailor,
    /// Decorative flower bed (Beauty +5).
    FlowerBed,
    /// Decorative statue (Beauty +10).
    Statue,
    /// Medical facility for healing.
    Hospital,
    /// Waste storage facility.
    Landfill,
    /// A place to bury corpses.
    Grave,
    /// Trading center for merchants.
    TradeDepot,
    /// Power generator (Energy).
    Generator,
    /// Power transmission pole (Energy).
    PowerPole,
    /// Basic wall for enclosure.
    Wall,
    /// Gate that can be opened/closed.
    Gate,
    /// Defensive tower.
    Tower,
    /// Ancient power generator (Ancient Structure).
    AncientReactor,
    /// Ancient manufacturing unit (Ancient Structure).
    AncientFabricator,
    /// Refines Ore into Fuel.
    Refinery,
}

impl BuildingType {
    /// Returns true if this building supports material variants.
    #[must_use]
    pub const fn supports_material(&self) -> bool {
        matches!(
            self,
            Self::Wall | Self::Gate | Self::Housing | Self::Statue | Self::Tower
        )
    }

    /// Returns true if this building blocks movement.
    #[must_use]
    pub const fn is_obstacle(&self) -> bool {
        !matches!(
            self,
            Self::Farm
                | Self::Stockpile
                | Self::Plantation
                | Self::FlowerBed
                | Self::Grave
                | Self::TradeDepot
                | Self::Landfill
        )
    }

    /// Returns the beauty value emitted by this building.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn beauty_value(&self) -> f32 {
        match self {
            Self::Statue => 10.0,
            Self::Landfill => -10.0,
            Self::Grave => -2.0, // Graves are slightly spooky
            Self::FlowerBed | Self::TradeDepot => 5.0, // Trade brings goods and culture
            Self::Well => 1.0,
            Self::Wall | Self::Gate | Self::Tower => 0.0,
            _ => 0.0,
        }
    }

    /// Returns the tech required to build this building, if any.
    #[must_use]
    pub const fn required_tech(&self) -> Option<Tech> {
        match self {
            Self::Smelter | Self::Smithy | Self::Generator | Self::PowerPole => {
                Some(Tech::MetalWorking)
            }
            Self::Tavern | Self::Statue => Some(Tech::SocialStructures),
            Self::Tower => Some(Tech::Masonry),
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
            Self::Well => "Well",
            Self::Stockpile => "Stockpile",
            Self::Smokehouse => "Smokehouse",
            Self::LumberMill => "Lumber Mill",
            Self::StoneMason => "Stone Mason",
            Self::Smelter => "Smelter",
            Self::Smithy => "Smithy",
            Self::Tavern => "Tavern",
            Self::Library => "Library",
            Self::Plantation => "Plantation",
            Self::Weaver => "Weaver",
            Self::Tailor => "Tailor",
            Self::FlowerBed => "Flower Bed",
            Self::Statue => "Statue",
            Self::Hospital => "Hospital",
            Self::Landfill => "Landfill",
            Self::Grave => "Grave",
            Self::TradeDepot => "Trade Depot",
            Self::Generator => "Generator",
            Self::PowerPole => "Power Pole",
            Self::Wall => "Wall",
            Self::Gate => "Gate",
            Self::Tower => "Tower",
            Self::AncientReactor => "Ancient Reactor",
            Self::AncientFabricator => "Ancient Fabricator",
            Self::Refinery => "Refinery",
        }
    }

    /// Returns the character representation of the building.
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Housing => 'H',
            Self::Farm | Self::AncientFabricator => 'F',
            Self::Well => 'U',
            Self::Stockpile => '=',
            Self::Smokehouse => '♨',
            Self::LumberMill => 'L',
            Self::StoneMason => 'M',
            Self::Smelter => 'S',
            Self::Smithy | Self::Tavern | Self::Tailor => 'T',
            Self::Library => '?', // Placeholder
            Self::Plantation => 'P',
            Self::Weaver => 'W',
            Self::FlowerBed => '*',
            Self::Statue => 'I',
            Self::Hospital | Self::Gate => '+',
            Self::Landfill => '%',
            Self::Grave => '†',
            Self::TradeDepot => '$',
            Self::Generator => 'G',
            Self::PowerPole => '|',
            Self::Wall => '#',
            Self::Tower => 'O',
            Self::AncientReactor | Self::Refinery => 'R',
        }
    }

    /// Returns the resource cost to build this building with the specified material.
    #[must_use]
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    pub const fn cost(&self, material: MaterialType) -> ColonyResources {
        match self {
            Self::Wall => match material {
                MaterialType::Wood => ColonyResources {
                    wood: 5.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Stone => ColonyResources {
                    stone: 5.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Metal => ColonyResources {
                    metal: 5.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Gold => ColonyResources {
                    metal: 50.0, // Gold is expensive (approximated as metal for now or free if we don't track gold?)
                    ..ColonyResources::zeroed()
                },
            },
            Self::Gate => match material {
                MaterialType::Wood => ColonyResources {
                    wood: 10.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Stone => ColonyResources {
                    stone: 10.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Metal => ColonyResources {
                    metal: 10.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Gold => ColonyResources {
                    metal: 100.0,
                    ..ColonyResources::zeroed()
                },
            },
            Self::Tower => ColonyResources {
                wood: 30.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Housing => match material {
                MaterialType::Wood => ColonyResources {
                    wood: 10.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Stone => ColonyResources {
                    stone: 10.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Metal => ColonyResources {
                    metal: 10.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Gold => ColonyResources {
                    metal: 100.0,
                    ..ColonyResources::zeroed()
                },
            },
            Self::Farm => ColonyResources {
                wood: 20.0,
                stone: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::Well => ColonyResources {
                wood: 5.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Stockpile => ColonyResources {
                wood: 50.0,
                ..ColonyResources::zeroed()
            },
            Self::Smokehouse => ColonyResources {
                wood: 30.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Landfill => ColonyResources {
                wood: 20.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::LumberMill | Self::Smithy => ColonyResources {
                wood: 30.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::StoneMason => ColonyResources {
                wood: 40.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::Smelter => ColonyResources {
                wood: 20.0,
                stone: 50.0,
                ..ColonyResources::zeroed()
            },
            Self::Tavern | Self::Hospital => ColonyResources {
                wood: 40.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Library => ColonyResources::zeroed(),
            Self::Plantation => ColonyResources {
                wood: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::Weaver | Self::Tailor => ColonyResources {
                wood: 30.0,
                stone: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::FlowerBed => ColonyResources {
                wood: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::Statue => match material {
                MaterialType::Stone | MaterialType::Wood => ColonyResources {
                    stone: 20.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Metal => ColonyResources {
                    metal: 20.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Gold => ColonyResources {
                    metal: 200.0,
                    ..ColonyResources::zeroed()
                },
            },
            Self::Grave => ColonyResources {
                stone: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::TradeDepot => ColonyResources {
                wood: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::Generator => ColonyResources {
                stone: 20.0,
                metal: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::PowerPole => ColonyResources {
                metal: 2.0,
                ..ColonyResources::zeroed()
            },
            Self::Refinery => ColonyResources {
                wood: 20.0,
                stone: 30.0,
                metal: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::AncientReactor | Self::AncientFabricator => ColonyResources::zeroed(),
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
    /// The currently selected material.
    pub selected_material: MaterialType,
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

/// Defines when a building operates.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShiftSchedule {
    /// Whether the building operates during the day (Dawn, Day, Dusk).
    pub day_shift: bool,
    /// Whether the building operates during the night.
    pub night_shift: bool,
}

impl Default for ShiftSchedule {
    fn default() -> Self {
        Self {
            day_shift: true,
            night_shift: false,
        }
    }
}

impl ShiftSchedule {
    /// Checks if the schedule is active for the given time of day.
    #[must_use]
    pub const fn is_active(&self, time: crate::layer1::day_night::TimeOfDay) -> bool {
        use crate::layer1::day_night::TimeOfDay;
        match time {
            TimeOfDay::Night => self.night_shift,
            _ => self.day_shift,
        }
    }
}

#[allow(clippy::too_many_lines, clippy::match_same_arms)]
fn spawn_building(
    world: &mut World,
    x: i32,
    y: i32,
    building_type: BuildingType,
    material: MaterialType,
) {
    let mut entity = world.spawn((
        Building { building_type },
        GridPosition { x, y },
        Material(material),
    ));

    // Calculate HP based on material
    let base_hp = 50.0;
    let max_hp = base_hp * material.hp_modifier();
    entity.insert(crate::layer1::structure::Structure {
        max_hp,
        current_hp: max_hp,
    });

    // Flammability
    if material.flammability() {
        entity.insert(Flammable::default());
    }

    // Beauty
    let base_beauty = building_type.beauty_value();
    let final_beauty = base_beauty + material.beauty_modifier();
    if final_beauty.abs() > f32::EPSILON {
        entity.insert(BeautySource {
            value: final_beauty,
            radius: 0.0,
        });
    }

    match building_type {
        BuildingType::Gate => {
            entity.insert(crate::layer1::defense::Gate::default());
        }
        BuildingType::Wall | BuildingType::Tower => {
            // Logic handled by generic material/structure above
        }
        BuildingType::Housing => {
            entity.insert((
                Housing::default(),
                LightSource {
                    radius: 3.0,
                    intensity: 0.5,
                    color: (255, 255, 100), // Yellow
                },
            ));
        }
        BuildingType::Farm | BuildingType::Plantation => {
            entity.insert((Farm::default(), ShiftSchedule::default()));
        }
        BuildingType::Well => {
            entity.insert(WaterSource {
                range: 5,
                amount: MAX_HYDRATION,
            });
        }
        BuildingType::Stockpile => {
            entity.insert(Stockpile::default());
        }
        BuildingType::Smokehouse => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    radius: 4.0,
                    intensity: 0.5,
                    color: (200, 200, 200), // Smoky white/grey
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Landfill => {
            entity.insert(Stockpile {
                waste_bonus: 100.0,
                food_bonus: 0.0,
                wood_bonus: 0.0,
                stone_bonus: 0.0,
            });
        }
        BuildingType::LumberMill => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    radius: 4.0,
                    intensity: 0.5,
                    color: (200, 180, 100), // Dim Wood light
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Weaver | BuildingType::Tailor => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Tavern => {
            entity.insert((
                Tavern::default(),
                LightSource {
                    radius: 8.0,
                    intensity: 0.8,
                    color: (255, 140, 0), // Orange
                },
            ));
        }
        BuildingType::Smelter => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    radius: 5.0,
                    intensity: 0.9,
                    color: (255, 50, 0), // Red/Fire
                },
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Smithy => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    radius: 4.0,
                    intensity: 0.7,
                    color: (255, 100, 0), // Orange/Fire
                },
                PowerConsumer {
                    demand: 2.0,
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::StoneMason => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Library => {
            entity.insert((
                Library,
                LightSource {
                    radius: 6.0,
                    intensity: 0.6,
                    color: (240, 240, 255), // White/Blueish
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::FlowerBed => {
            // Flammability handled by material (likely wood/plant based for flower bed?)
            // If FlowerBed is technically "Wood" (default), it's flammable.
            // If we want it to always be flammable regardless of "Material" (because plants burn),
            // we should force it.
            entity.insert(Flammable::default());
        }
        BuildingType::Statue => {
            // Statues are made of stone/metal, not flammable
        }
        BuildingType::Generator => {
            entity.insert(PowerSource { output: 10.0 });
        }
        BuildingType::PowerPole => {
            entity.insert(Conduit);
        }
        BuildingType::Hospital => {
            entity.insert((
                crate::layer1::medical::Hospital::default(),
                LightSource {
                    radius: 6.0,
                    intensity: 0.7,
                    color: (255, 255, 255), // Pure White
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Grave => {
            entity.insert(crate::layer1::funeral::Grave::default());
        }
        BuildingType::TradeDepot => {
            entity.insert((
                TradeDepot,
                LightSource {
                    radius: 5.0,
                    intensity: 0.6,
                    color: (220, 220, 100), // Yellowish
                },
            ));
        }
        BuildingType::AncientReactor => {
            entity.insert((
                PowerSource { output: 50.0 }, // Massive power
                AncientStructure,
                LightSource {
                    radius: 8.0,
                    intensity: 1.0,
                    color: (255, 215, 0), // Gold
                },
            ));
            // Set high HP
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 1000.0;
                structure.current_hp = 1000.0;
            }
        }
        BuildingType::AncientFabricator => {
            entity.insert((
                // Refining logic needs to be added, maybe RefiningProgress with high speed?
                // For now, just mark it.
                RefiningProgress {
                    current: 0.0,
                    max: 1.0, // Very fast? Default is 10.0
                },
                AncientStructure,
                LightSource {
                    radius: 6.0,
                    intensity: 0.8,
                    color: (0, 255, 255), // Cyan
                },
                ShiftSchedule::default(),
            ));
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 1000.0;
                structure.current_hp = 1000.0;
            }
        }
        BuildingType::Refinery => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 20.0, // Slower process
                },
                LightSource {
                    radius: 6.0,
                    intensity: 0.8,
                    color: (100, 200, 255), // Chemical blue
                },
                ShiftSchedule::default(),
            ));
        }
    }
}

/// Helper for spawning buildings in tests/tools.
pub fn spawn_building_with_material(
    world: &mut World,
    x: i32,
    y: i32,
    building_type: BuildingType,
    material: MaterialType,
) {
    spawn_building(world, x, y, building_type, material);
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

    // Get material
    let material = if building_type.supports_material() {
        world
            .get_resource::<BuildMode>()
            .map(|m| m.selected_material)
            .unwrap_or_default()
    } else {
        MaterialType::default()
    };

    // Check affordability and deduct cost
    let cost = building_type.cost(material);
    let can_afford = world.resource_mut::<ColonyResources>().try_deduct(&cost);

    if !can_afford {
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add(format!(
                "Not enough resources for {}",
                building_type.label()
            ));
        }
        return false;
    }

    // Spawn building
    spawn_building(world, x, y, building_type, material);

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
        assert_eq!(BuildingType::Farm.next(), BuildingType::Well);
        assert_eq!(BuildingType::Well.next(), BuildingType::Stockpile);
        assert_eq!(BuildingType::Stockpile.next(), BuildingType::Smokehouse);
        assert_eq!(BuildingType::Smokehouse.next(), BuildingType::LumberMill);
        assert_eq!(BuildingType::LumberMill.next(), BuildingType::StoneMason);
        assert_eq!(BuildingType::StoneMason.next(), BuildingType::Smelter);
        assert_eq!(BuildingType::Smelter.next(), BuildingType::Smithy);
        assert_eq!(BuildingType::Smithy.next(), BuildingType::Tavern);
        assert_eq!(BuildingType::Tavern.next(), BuildingType::Library);
        assert_eq!(BuildingType::Library.next(), BuildingType::Plantation);
        assert_eq!(BuildingType::Plantation.next(), BuildingType::Weaver);
        assert_eq!(BuildingType::Weaver.next(), BuildingType::Tailor);
        assert_eq!(BuildingType::Tailor.next(), BuildingType::FlowerBed);
        assert_eq!(BuildingType::FlowerBed.next(), BuildingType::Statue);
        assert_eq!(BuildingType::Statue.next(), BuildingType::Hospital);
        assert_eq!(BuildingType::Hospital.next(), BuildingType::Landfill);
        assert_eq!(BuildingType::Landfill.next(), BuildingType::Grave);
        assert_eq!(BuildingType::Grave.next(), BuildingType::TradeDepot);
        assert_eq!(BuildingType::TradeDepot.next(), BuildingType::Generator);
        assert_eq!(BuildingType::Generator.next(), BuildingType::PowerPole);
        assert_eq!(BuildingType::PowerPole.next(), BuildingType::Wall);
        assert_eq!(BuildingType::Wall.next(), BuildingType::Gate);
        assert_eq!(BuildingType::Gate.next(), BuildingType::Tower);
        assert_eq!(BuildingType::Tower.next(), BuildingType::AncientReactor);
        assert_eq!(
            BuildingType::AncientReactor.next(),
            BuildingType::AncientFabricator
        );
        assert_eq!(
            BuildingType::AncientFabricator.next(),
            BuildingType::Refinery
        );
        assert_eq!(BuildingType::Refinery.next(), BuildingType::Housing);
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
        assert_eq!(mode.selected, BuildingType::Well);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Stockpile);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Smokehouse);

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
        assert_eq!(mode.selected, BuildingType::Plantation);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Weaver);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Tailor);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::FlowerBed);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Statue);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Hospital);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Landfill);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Grave);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::TradeDepot);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Generator);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::PowerPole);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Wall);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Gate);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Tower);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AncientReactor);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AncientFabricator);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Refinery);

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
        let cost = BuildingType::Housing.cost(MaterialType::Wood);
        assert!((cost.wood - 10.0).abs() < f32::EPSILON);
        assert!((cost.stone - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_farm_cost() {
        let cost = BuildingType::Farm.cost(MaterialType::default());
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

    #[test]
    fn test_place_building_adds_structure() {
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

        let structure_count = world
            .query::<&crate::layer1::structure::Structure>()
            .iter(&world)
            .count();
        assert_eq!(structure_count, 1, "Should have added Structure component");
    }

    #[test]
    fn test_place_gate_adds_gate_component() {
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

        try_place_building(&mut world, 5, 5, BuildingType::Gate);

        let gate_count = world
            .query::<&crate::layer1::defense::Gate>()
            .iter(&world)
            .count();
        assert_eq!(gate_count, 1, "Should have added Gate component");
    }
}

#[cfg(test)]
mod shift_tests {
    use super::*;
    use crate::layer1::day_night::TimeOfDay;

    #[test]
    fn test_shift_schedule_default() {
        // Default behavior: Day shift enabled, Night shift disabled
        let schedule = ShiftSchedule::default();
        assert!(schedule.day_shift, "Day shift should be enabled by default");
        assert!(
            !schedule.night_shift,
            "Night shift should be disabled by default"
        );
    }

    #[test]
    fn test_shift_active_during_day() {
        let schedule = ShiftSchedule {
            day_shift: true,
            night_shift: false,
        };

        // Day shifts cover Dawn, Day, and Dusk
        assert!(
            schedule.is_active(TimeOfDay::Dawn),
            "Should be active at Dawn"
        );
        assert!(
            schedule.is_active(TimeOfDay::Day),
            "Should be active at Day"
        );
        assert!(
            schedule.is_active(TimeOfDay::Dusk),
            "Should be active at Dusk"
        );
        assert!(
            !schedule.is_active(TimeOfDay::Night),
            "Should NOT be active at Night"
        );
    }

    #[test]
    fn test_shift_active_during_night() {
        let schedule = ShiftSchedule {
            day_shift: false,
            night_shift: true,
        };

        assert!(
            !schedule.is_active(TimeOfDay::Dawn),
            "Should NOT be active at Dawn"
        );
        assert!(
            !schedule.is_active(TimeOfDay::Day),
            "Should NOT be active at Day"
        );
        assert!(
            !schedule.is_active(TimeOfDay::Dusk),
            "Should NOT be active at Dusk"
        );
        assert!(
            schedule.is_active(TimeOfDay::Night),
            "Should be active at Night"
        );
    }

    #[test]
    fn test_shift_active_always() {
        let schedule = ShiftSchedule {
            day_shift: true,
            night_shift: true,
        };

        assert!(schedule.is_active(TimeOfDay::Dawn));
        assert!(schedule.is_active(TimeOfDay::Day));
        assert!(schedule.is_active(TimeOfDay::Dusk));
        assert!(schedule.is_active(TimeOfDay::Night));
    }
}
