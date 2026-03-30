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

use super::acoustic::NoiseSource;
use super::beauty::BeautySource;
use super::farm::Farm;
use super::fire::Flammable;
use super::housing::Housing;
use super::social::Tavern;
use super::stockpile::Stockpile;
use super::GridPosition;
use crate::layer1::access_control::AccessControl;
use crate::layer1::admin::{AdminConsumer, AdminProvider, Office};
use crate::layer1::ai_core::AICore;
use crate::layer1::atmosphere::CorrosionResistant;
use crate::layer1::control::DoorControl;
use crate::layer1::drone::DroneHub;
use crate::layer1::energy::{Conduit, FuelConsumer, PowerConsumer, PowerSource};
use crate::layer1::heirloom::AncientStructure;
use crate::layer1::inventory::Inventory;
use crate::layer1::items::ItemType;
use crate::layer1::lighting::LightSource;
use crate::layer1::permit::PermitRequired;
use crate::layer1::prototyping::{BuildingMastery, Prototype};
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use crate::layer1::rituals::MachineSpirit;
use crate::layer1::seismic::SeismicSource;
use crate::layer1::solar::SolarPower;
use crate::layer1::tech::{DataStorage, Library, Tech, TechState};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::trade::TradeDepot;
use crate::layer1::water::{WaterSource, MAX_HYDRATION};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use bevy_ecs::world::EntityWorldMut;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Direction for buildings (e.g., Conveyor Belts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    #[default]
    /// North direction (0, -1).
    North,
    /// East direction (1, 0).
    East,
    /// South direction (0, 1).
    South,
    /// West direction (-1, 0).
    West,
}

impl Direction {
    /// Returns the vector representation of the direction (dx, dy).
    #[must_use]
    pub const fn to_delta(&self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }
}

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

/// Building tech tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Basic, primitive technology.
    Basic = 1,
    /// Advanced, industrial technology.
    Advanced = 2,
    /// High-tech, futuristic or ancient technology.
    HighTech = 3,
}

/// Building category for tech comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Related to food (Farms, Hydroponics).
    FoodProduction,
    /// Related to material processing (Smelters, Refineries).
    Manufacturing,
    /// Related to energy generation (Generators, Reactors).
    Power,
    /// Related to science and data (Libraries, AI Cores).
    Research,
}

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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, EnumIter)]
pub enum BuildingType {
    /// Basic shelter for pops.
    #[default]
    Housing,
    /// Office for administration.
    Office,
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
    /// Solar Panel power source.
    SolarPanel,
    /// Power transmission pole (Energy).
    PowerPole,
    /// Battery for energy storage.
    Battery,
    /// Basic wall for enclosure.
    Wall,
    /// Window that allows viewing outside.
    Window,
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
    /// Specialized farm that works in Winter.
    Greenhouse,
    /// Small personal storage shed (Spontaneous Architecture).
    PersonalShed,
    /// Small personal garden (Spontaneous Architecture).
    PersonalGarden,
    /// Small personal shrine (Spontaneous Architecture).
    PersonalShrine,
    /// Research center for space observation.
    Observatory,
    /// Logistics: Moves items.
    ConveyorBelt,
    /// Logistics: Collects items into global storage.
    Hopper,
    /// Hydroponics Bay: Grows food using water and power.
    HydroponicsBay,
    /// Generates atmospheric pressure.
    LifeSupport,
    /// Maintains pressure while allowing passage.
    Airlock,
    /// Allows gases to pass freely while maintaining physical security.
    Vent,
    /// Defensive structure that consumes Waste as ammunition.
    TrashCannon,
    /// Generates heat to combat cold temperatures.
    Heater,
    /// Stores data capacity for technology.
    ServerBank,
    /// The colony's starting ship (can be cannibalized for resources).
    Lander,
    /// Command Center providing system visibility.
    CommandCenter,
    /// High-tech AI Core for base automation.
    AICore,
    /// Hub for spawning and recharging Drones.
    DroneHub,
    /// Cryo-Stasis Pod.
    CryoPod,
    /// Harvests energy from magnetic storms.
    AuroralCollector,
    /// Terraforming: Atmospheric Processor.
    AtmosphericProcessor,
    /// Stores genetic samples of flora and fauna.
    GeneBank,
    /// Produces Pops from Rations and Energy.
    CloneVat,
    /// Hypno-Learning Pod (Spec 255).
    HypnoPod,
    /// Facility for pops to clean themselves (consumes Water).
    Shower,
    /// Converts Waste and Corpses into Rations.
    Recycler,
    /// A place for pops to post grievances and commendations.
    BulletinBoard,
    /// Holographic projector that emits Beauty when powered.
    HoloProjector,
}

impl BuildingType {
    /// Returns the tech tier and category for this building, if applicable.
    /// Used for Tech Envy (Spec 162).
    #[must_use]
    pub const fn tier_info(&self) -> Option<(Category, Tier)> {
        match self {
            Self::Farm | Self::Recycler => Some((Category::FoodProduction, Tier::Basic)),
            Self::Greenhouse => Some((Category::FoodProduction, Tier::Advanced)),
            Self::HydroponicsBay => Some((Category::FoodProduction, Tier::HighTech)),

            Self::Smithy | Self::LumberMill | Self::StoneMason => {
                Some((Category::Manufacturing, Tier::Basic))
            }
            Self::Smelter | Self::Refinery => Some((Category::Manufacturing, Tier::Advanced)),
            Self::AncientFabricator => Some((Category::Manufacturing, Tier::HighTech)),

            Self::Generator | Self::SolarPanel => Some((Category::Power, Tier::Basic)),
            Self::AuroralCollector => Some((Category::Power, Tier::Advanced)),
            Self::AncientReactor => Some((Category::Power, Tier::HighTech)),

            Self::Library | Self::BulletinBoard => Some((Category::Research, Tier::Basic)),
            Self::Observatory | Self::CryoPod => Some((Category::Research, Tier::Advanced)),
            Self::AICore
            | Self::AtmosphericProcessor
            | Self::GeneBank
            | Self::CloneVat
            | Self::HypnoPod
            | Self::HoloProjector => Some((Category::Research, Tier::HighTech)),

            _ => None,
        }
    }

    /// Returns the thermal conductivity (0.0 to 1.0) of the building.
    /// Lower values mean better insulation.
    /// - 1.0: Passes heat freely (Vent, Empty)
    /// - 0.05: Good insulation (Wall)
    #[must_use]
    pub const fn thermal_conductivity(&self) -> f32 {
        match self {
            Self::Wall => 0.05,
            Self::Window | Self::Airlock => 0.1, // Windows/Airlocks insulate well but leak
            Self::Gate => 0.5,                   // Gates are less insulated than walls
            _ => 1.0, // Most buildings don't block heat flow significantly
        }
    }

    /// Returns the thermal retention (0.0 to 1.0) of the building (Spec 198).
    /// Higher values mean the building holds heat/cold longer (Thermal Mass).
    /// - 0.8: High Mass (Wall, Tower)
    /// - 0.6: Medium Mass (Housing, Office)
    /// - 0.1: Low Mass (`FlowerBed`)
    #[must_use]
    pub const fn heat_retention(&self) -> f32 {
        match self {
            Self::Wall | Self::Tower | Self::AncientReactor | Self::AncientFabricator => 0.8,
            Self::Housing
            | Self::Office
            | Self::Stockpile
            | Self::LumberMill
            | Self::StoneMason
            | Self::Smelter
            | Self::Smithy
            | Self::Tavern
            | Self::Hospital
            | Self::CommandCenter
            | Self::AICore
            | Self::Recycler => 0.6,
            Self::FlowerBed | Self::PersonalGarden | Self::Grave | Self::BulletinBoard => 0.1,
            _ => 0.5,
        }
    }

    /// Returns true if this building supports material variants.
    #[must_use]
    pub const fn supports_material(&self) -> bool {
        matches!(
            self,
            Self::Wall | Self::Gate | Self::Housing | Self::Statue | Self::Tower | Self::Airlock
        )
    }

    /// Returns the flow transmissivity (0.0 to 1.0) for atmospheric simulation.
    /// Returns `None` if the building does not affect flow (treat as 1.0).
    #[must_use]
    pub const fn flow_transmissivity(&self) -> Option<f32> {
        match self {
            Self::Wall | Self::Window | Self::Airlock => Some(0.0),
            Self::Gate => Some(0.5),
            Self::Vent => Some(1.0),
            _ => None,
        }
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
                | Self::PersonalGarden
                | Self::ConveyorBelt
                | Self::Airlock // Vent is explicitly an obstacle for standard movement (blocks Pops),
                                // but Vermin can pass through it (handled in pathfinding).
                                // So here it returns true (is obstacle).
        )
    }

    /// Returns true if this building blocks wind flow.
    #[must_use]
    pub const fn blocks_wind(&self) -> bool {
        match self {
            // Walls and large structures
            Self::Wall
            | Self::Window
            | Self::Gate
            | Self::Tower
            | Self::Housing
            | Self::Office
            | Self::Lander
            | Self::CommandCenter
            | Self::AICore
            | Self::DroneHub
            | Self::CryoPod
            | Self::Smokehouse
            | Self::LumberMill
            | Self::StoneMason
            | Self::Smelter
            | Self::Smithy
            | Self::Weaver
            | Self::Tailor
            | Self::Refinery
            | Self::Greenhouse
            | Self::ServerBank
            | Self::Tavern
            | Self::Library
            | Self::Hospital
            | Self::Observatory
            | Self::Battery
            | Self::LifeSupport
            | Self::Airlock
            | Self::AncientReactor
            | Self::AncientFabricator
            | Self::AtmosphericProcessor
            | Self::GeneBank
            | Self::CloneVat
            | Self::HypnoPod
            | Self::Shower => true,

            // Small or Open structures
            Self::Farm
            | Self::Well
            | Self::Stockpile
            | Self::Plantation
            | Self::FlowerBed
            | Self::Statue
            | Self::Landfill
            | Self::Grave
            | Self::TradeDepot
            | Self::Generator
            | Self::SolarPanel
            | Self::PowerPole
            | Self::PersonalShed
            | Self::PersonalGarden
            | Self::PersonalShrine
            | Self::ConveyorBelt
            | Self::Hopper
            | Self::HydroponicsBay
            | Self::Vent
            | Self::TrashCannon
            | Self::Heater
            | Self::AuroralCollector => false,
            Self::Recycler => true,
            Self::BulletinBoard => false,
            Self::HoloProjector => false,
        }
    }

    /// Returns true if this building is immune to seasonal penalties (e.g., Winter food penalty).
    #[must_use]
    pub const fn seasonal_immunity(&self) -> bool {
        matches!(self, Self::Greenhouse | Self::HydroponicsBay)
    }

    /// Returns the beauty value emitted by this building.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn beauty_value(&self) -> f32 {
        match self {
            Self::Statue => super::beauty::STATUE_BEAUTY,
            Self::Landfill => -10.0,
            Self::Recycler => -5.0, // Grim machinery
            Self::Grave => -2.0,    // Graves are slightly spooky
            Self::FlowerBed => super::beauty::FLOWER_BED_BEAUTY,
            Self::HoloProjector => 50.0, // Massive beauty boost
            Self::TradeDepot => 5.0,     // Trade brings goods and culture
            Self::Well | Self::HydroponicsBay | Self::LifeSupport => 1.0,
            Self::Wall | Self::Window | Self::Gate | Self::Tower | Self::Airlock | Self::Vent => {
                0.0
            }
            Self::TrashCannon => -2.0, // Industrial machinery is ugly
            Self::Heater | Self::ServerBank => 0.0,
            Self::CommandCenter | Self::AICore | Self::CryoPod | Self::GeneBank => 0.0,
            Self::CloneVat => -5.0, // Unsettling
            Self::HypnoPod => -2.0, // Unsettling
            _ => 0.0,
        }
    }

    /// Returns the radius of beauty effect.
    /// Most buildings are 0.0 (single tile).
    #[must_use]
    pub const fn beauty_radius(&self) -> f32 {
        match self {
            Self::Statue => 5.0,
            Self::FlowerBed => 3.0,
            Self::Landfill => 8.0,
            Self::Recycler => 4.0,
            Self::TradeDepot => 4.0,
            Self::Grave | Self::Well | Self::HydroponicsBay | Self::LifeSupport => 2.0,
            Self::CloneVat => 3.0,
            Self::HypnoPod => 2.0,
            Self::HoloProjector => 8.0,
            _ => 0.0,
        }
    }

    /// Returns the tech required to build this building, if any.
    #[must_use]
    pub const fn required_tech(&self) -> Option<Tech> {
        match self {
            Self::Smelter
            | Self::Smithy
            | Self::Generator
            | Self::SolarPanel
            | Self::PowerPole
            | Self::Battery
            | Self::ConveyorBelt
            | Self::Hopper
            | Self::LifeSupport
            | Self::Airlock
            | Self::Vent
            | Self::Heater
            | Self::ServerBank
            | Self::CommandCenter
            | Self::AICore => Some(Tech::MetalWorking),
            Self::Tavern | Self::Statue => Some(Tech::SocialStructures),
            Self::Tower => Some(Tech::Masonry),
            Self::Observatory => Some(Tech::Astronomy),
            Self::HydroponicsBay => Some(Tech::Hydroponics),
            Self::TrashCannon => Some(Tech::Militia),
            Self::CryoPod => Some(Tech::Medical),
            Self::AuroralCollector => Some(Tech::Electromagnetism),
            Self::AtmosphericProcessor => Some(Tech::Terraforming),
            Self::GeneBank | Self::CloneVat | Self::HypnoPod => Some(Tech::Medical),
            Self::Shower => Some(Tech::SocialStructures),
            Self::Recycler => Some(Tech::Medical),
            Self::BulletinBoard => Some(Tech::SocialStructures),
            Self::HoloProjector => Some(Tech::Electromagnetism), // Assumed tech
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
            Self::Office => "Office",
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
            Self::SolarPanel => "Solar Panel",
            Self::PowerPole => "Power Pole",
            Self::Battery => "Battery",
            Self::Wall => "Wall",
            Self::Window => "Window",
            Self::Gate => "Gate",
            Self::Tower => "Tower",
            Self::AncientReactor => "Ancient Reactor",
            Self::AncientFabricator => "Ancient Fabricator",
            Self::Refinery => "Refinery",
            Self::Greenhouse => "Greenhouse",
            Self::PersonalShed => "Shed",
            Self::PersonalGarden => "Garden",
            Self::PersonalShrine => "Shrine",
            Self::Observatory => "Observatory",
            Self::ConveyorBelt => "Conveyor Belt",
            Self::Hopper => "Hopper",
            Self::HydroponicsBay => "Hydroponics Bay",
            Self::LifeSupport => "Life Support",
            Self::Airlock => "Airlock",
            Self::Vent => "Vent",
            Self::TrashCannon => "Trash Cannon",
            Self::Heater => "Heater",
            Self::ServerBank => "Server Bank",
            Self::Lander => "Lander",
            Self::CommandCenter => "Command Center",
            Self::AICore => "AI Core",
            Self::DroneHub => "Drone Hub",
            Self::CryoPod => "Cryo Pod",
            Self::AuroralCollector => "Auroral Collector",
            Self::AtmosphericProcessor => "Atmospheric Processor",
            Self::GeneBank => "Gene Bank",
            Self::CloneVat => "Clone Vat",
            Self::HypnoPod => "Hypno-Pod",
            Self::Shower => "Shower",
            Self::Recycler => "Recycler",
            Self::BulletinBoard => "Bulletin Board",
            Self::HoloProjector => "Holo Projector",
        }
    }

    /// Returns the character representation of the building.
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Housing => 'H',
            Self::Office | Self::Tower | Self::Observatory | Self::HoloProjector => 'O',
            Self::Farm | Self::AncientFabricator => 'F',
            Self::HydroponicsBay => 'Y',
            Self::DroneHub => 'D',
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
            Self::FlowerBed | Self::PersonalGarden => '*',
            Self::Statue => 'I',
            Self::Hospital | Self::Gate => '+',
            Self::Landfill => '%',
            Self::Grave => '†',
            Self::TradeDepot => '$',
            Self::Generator | Self::Greenhouse => 'G',
            Self::SolarPanel => '☼',
            Self::PowerPole => '|',
            Self::Battery => 'B',
            Self::Wall => '#',
            Self::Window => '□',
            Self::AncientReactor | Self::Refinery => 'R',
            Self::PersonalShed => 's',
            Self::PersonalShrine => '☗',
            Self::ConveyorBelt => '>',
            Self::Hopper => 'V',
            Self::LifeSupport => '♼',
            Self::Airlock => '⌷',
            Self::Vent => '≡',
            Self::TrashCannon => '♣',
            Self::Heater => 'h',
            Self::ServerBank => '▥',
            Self::Lander => 'Λ',
            Self::CommandCenter => 'C',
            Self::AICore => 'A',
            Self::CryoPod => '❄',
            Self::AuroralCollector => 'Ψ',
            Self::AtmosphericProcessor => '@',
            Self::GeneBank => '🧬',
            Self::CloneVat => '⚗',
            Self::HypnoPod => 'H',
            Self::Shower => '🚿',
            Self::Recycler => '♻',
            Self::BulletinBoard => 'B',
        }
    }

    /// Returns the resource cost to build this building with the specified material.
    #[must_use]
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    pub const fn cost(&self, material: MaterialType) -> ColonyResources {
        match self {
            Self::AICore => ColonyResources {
                metal: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::DroneHub => ColonyResources {
                metal: 30.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::CryoPod => ColonyResources {
                metal: 20.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::AuroralCollector => ColonyResources {
                metal: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::AtmosphericProcessor => ColonyResources {
                metal: 200.0,
                stone: 100.0,
                ..ColonyResources::zeroed()
            },
            Self::GeneBank => ColonyResources {
                metal: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::CloneVat => ColonyResources {
                metal: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::HypnoPod => ColonyResources {
                metal: 100.0,
                tools: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::Shower => ColonyResources {
                metal: 10.0,
                stone: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::Recycler => ColonyResources {
                metal: 20.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::BulletinBoard => ColonyResources {
                wood: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::HoloProjector => ColonyResources {
                metal: 20.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::CommandCenter => ColonyResources {
                metal: 50.0,
                stone: 50.0,
                ..ColonyResources::zeroed()
            },
            Self::ServerBank => ColonyResources {
                metal: 20.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::TrashCannon => ColonyResources {
                metal: 20.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Heater => ColonyResources {
                metal: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::LifeSupport => ColonyResources {
                metal: 50.0,
                ..ColonyResources::zeroed()
            },
            Self::Airlock => match material {
                MaterialType::Wood => ColonyResources {
                    wood: 15.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Stone => ColonyResources {
                    stone: 15.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Metal => ColonyResources {
                    metal: 15.0,
                    ..ColonyResources::zeroed()
                },
                MaterialType::Gold => ColonyResources {
                    metal: 150.0,
                    ..ColonyResources::zeroed()
                },
            },
            Self::Vent => ColonyResources {
                metal: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::ConveyorBelt => ColonyResources {
                metal: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::Hopper => ColonyResources {
                metal: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::HydroponicsBay => ColonyResources {
                metal: 30.0,
                stone: 20.0, // 10 Stone + 10 Glass fallback
                ..ColonyResources::zeroed()
            },
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
            Self::Window => ColonyResources {
                wood: 5.0,
                ..ColonyResources::zeroed()
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
            Self::Office => ColonyResources {
                wood: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
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
            Self::SolarPanel => ColonyResources {
                metal: 10.0,
                stone: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::PowerPole => ColonyResources {
                metal: 2.0,
                ..ColonyResources::zeroed()
            },
            Self::Battery => ColonyResources {
                metal: 10.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Refinery => ColonyResources {
                wood: 20.0,
                stone: 30.0,
                metal: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::AncientReactor | Self::AncientFabricator => ColonyResources::zeroed(),
            Self::Greenhouse => ColonyResources {
                wood: 10.0,
                stone: 20.0,
                metal: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::PersonalShed => ColonyResources {
                wood: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::PersonalGarden => ColonyResources {
                wood: 5.0,
                ..ColonyResources::zeroed()
            },
            Self::PersonalShrine => ColonyResources {
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            Self::Observatory => ColonyResources {
                wood: 20.0,
                stone: 50.0,
                metal: 10.0, // Needs advanced materials
                ..ColonyResources::zeroed()
            },
            Self::Lander => ColonyResources::zeroed(),
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
    /// assert_eq!(BuildingType::Housing.next(), BuildingType::Office);
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
#[derive(Component, Default)]
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

/// A spatial map of buildings for fast lookup (Pos -> Entity).
#[derive(Resource, Default)]
pub struct BuildingMap(pub HashMap<(i32, i32), Entity>);

/// System to update the spatial building map.
///
/// This rebuilds the map every frame to ensure pathfinding has fresh data.
/// It avoids iterating all entities during pathfinding calls.
pub fn update_building_map_system(
    mut map: ResMut<BuildingMap>,
    query: Query<(Entity, &GridPosition), With<Building>>,
) {
    map.0.clear();
    for (entity, pos) in query.iter() {
        map.0.insert((pos.x, pos.y), entity);
    }
}

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

fn insert_base_building_components(
    entity: &mut EntityWorldMut<'_>,
    building_type: BuildingType,
    material: MaterialType,
) {
    // Calculate HP based on material
    let base_hp = 50.0;
    let max_hp = base_hp * material.hp_modifier();
    entity.insert(crate::layer1::structure::Structure {
        max_hp,
        current_hp: max_hp,
    });

    // Permit System: Advanced buildings require a permit
    if let Some((_, tier)) = building_type.tier_info() {
        if tier >= Tier::Advanced {
            entity.insert(PermitRequired);
            // Ensure inventory exists to accept permit
            entity.insert(Inventory::default());
        }
    }

    // Flammability
    if material.flammability() {
        entity.insert(Flammable::default());
    }

    // Beauty
    let base_beauty = building_type.beauty_value();
    let final_beauty = base_beauty + material.beauty_modifier();
    if final_beauty.abs() > f32::EPSILON {
        let radius = building_type.beauty_radius();
        entity.insert(BeautySource {
            value: final_beauty,
            radius,
        });
    }

    // Admin Consumer (All buildings consume admin)
    // Default 1.0, maybe scale by tier later?
    entity.insert(AdminConsumer { demand: 1.0 });

    // Corrosion Resistance based on Material
    match material {
        MaterialType::Stone | MaterialType::Metal => {
            entity.insert(CorrosionResistant { factor: 0.5 });
        }
        MaterialType::Gold => {
            entity.insert(CorrosionResistant { factor: 1.0 });
        }
        MaterialType::Wood => {
            // Wood rots, so no resistance (0.0)
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
) -> Entity {
    // Prototyping Phase: Check mastery before mutable borrow
    let is_mastered = world
        .get_resource::<BuildingMastery>()
        .is_none_or(|m| m.is_mastered(building_type));

    let mut entity = world.spawn((
        Building { building_type },
        GridPosition { x, y },
        Material(material),
    ));

    if !is_mastered {
        entity.insert(Prototype::default());
    }

    insert_base_building_components(&mut entity, building_type, material);

    match building_type {
        BuildingType::Office => {
            entity.insert((
                // Office provides admin
                AdminProvider { amount: 10.0 },
                Office::default(),
                // Office typically operates during the day
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Housing | BuildingType::Lander => {
            configure_housing(&mut entity, building_type);
        }
        BuildingType::Farm
        | BuildingType::Plantation
        | BuildingType::Greenhouse
        | BuildingType::HydroponicsBay
        | BuildingType::Smokehouse
        | BuildingType::LumberMill
        | BuildingType::StoneMason
        | BuildingType::Smelter
        | BuildingType::Smithy
        | BuildingType::Weaver
        | BuildingType::Tailor
        | BuildingType::Refinery
        | BuildingType::AncientFabricator => configure_production(&mut entity, building_type),
        BuildingType::Stockpile | BuildingType::Landfill => {
            configure_storage(&mut entity, building_type);
        }
        BuildingType::Tavern
        | BuildingType::Library
        | BuildingType::FlowerBed
        | BuildingType::Statue
        | BuildingType::Hospital
        | BuildingType::Grave
        | BuildingType::TradeDepot => configure_civic(&mut entity, building_type),
        BuildingType::Wall
        | BuildingType::Window
        | BuildingType::Gate
        | BuildingType::Tower
        | BuildingType::Well
        | BuildingType::ConveyorBelt
        | BuildingType::Hopper
        | BuildingType::Airlock
        | BuildingType::Vent => configure_infrastructure(&mut entity, building_type),
        BuildingType::Generator
        | BuildingType::SolarPanel
        | BuildingType::PowerPole
        | BuildingType::Battery
        | BuildingType::AncientReactor
        | BuildingType::Heater
        | BuildingType::AuroralCollector => configure_power(&mut entity, building_type),
        BuildingType::Observatory
        | BuildingType::LifeSupport
        | BuildingType::TrashCannon
        | BuildingType::ServerBank
        | BuildingType::CommandCenter
        | BuildingType::AICore
        | BuildingType::DroneHub
        | BuildingType::CryoPod
        | BuildingType::AtmosphericProcessor
        | BuildingType::GeneBank
        | BuildingType::CloneVat
        | BuildingType::HypnoPod
        | BuildingType::HoloProjector => configure_tech(&mut entity, building_type),
        BuildingType::Shower => configure_civic(&mut entity, building_type),
        BuildingType::Recycler => {
            // Recycler configuration
            entity.insert((
                crate::layer1::recycling::Recycler::default(),
                Inventory::default(),
                crate::layer1::lighting::LightSource {
                    is_outdoor: true,
                    radius: 3.0,
                    intensity: 0.5,
                    color: (0, 255, 0), // Green glow
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::BulletinBoard => {
            entity.insert((
                crate::layer1::social::grievances::BulletinBoard::default(),
                ShiftSchedule::default(),
            ));
        }
        BuildingType::PersonalShed
        | BuildingType::PersonalGarden
        | BuildingType::PersonalShrine => {
            // Logic handled by components added in system
        }
    }

    entity.id()
}

fn configure_housing(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Housing => {
            entity.insert((
                Housing::default(),
                LightSource {
                    is_outdoor: true,
                    radius: 3.0,
                    intensity: 0.5,
                    color: (255, 255, 100), // Yellow
                },
            ));
        }
        BuildingType::Lander => {
            entity.insert((
                Housing {
                    capacity: 5,
                    ..Default::default()
                },
                Stockpile {
                    food_bonus: 50.0,
                    wood_bonus: 50.0,
                    stone_bonus: 20.0,
                    waste_bonus: 0.0,
                },
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 8.0,
                    intensity: 0.8,
                    color: (200, 200, 255),
                },
                // Spec Q&A says Library/Lander should provide base capacity.
                // I will add DataStorage to Lander too!
                DataStorage { capacity: 10.0 }, // Base capacity
            ));
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 500.0;
                structure.current_hp = 500.0;
            }
        }
        _ => {}
    }
}

fn configure_farm_buildings(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Farm | BuildingType::Greenhouse => {
            let mut rng = rand::thread_rng();
            let crops = [
                ItemType::Potato,
                ItemType::Wheat,
                ItemType::Rice,
                ItemType::Corn,
                ItemType::Soy,
            ];
            let crop_type = crops.choose(&mut rng).cloned().unwrap_or(ItemType::Potato);
            entity.insert((
                Farm {
                    selected_crop: crop_type,
                    ..Default::default()
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Plantation => {
            entity.insert((Farm::default(), ShiftSchedule::default()));
        }
        BuildingType::HydroponicsBay => {
            let mut rng = rand::thread_rng();
            let crops = [ItemType::Rice, ItemType::Soy];
            let crop_type = crops.choose(&mut rng).cloned().unwrap_or(ItemType::Rice);
            entity.insert((
                Farm {
                    selected_crop: crop_type,
                    ..Default::default()
                },
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        _ => {}
    }
}

#[allow(clippy::too_many_lines)]
fn configure_refining_buildings(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Smokehouse => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.5,
                    color: (200, 200, 200), // Smoky white/grey
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::LumberMill => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.5,
                    color: (200, 180, 100), // Dim Wood light
                },
                SeismicSource {
                    intensity: 0.5,
                    radius: 3.0,
                },
                NoiseSource {
                    radius: 6.0,
                    intensity: 0.8,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Smelter => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.9,
                    color: (255, 50, 0), // Red/Fire
                },
                SeismicSource {
                    intensity: 0.5,
                    radius: 3.0,
                },
                NoiseSource {
                    radius: 8.0,
                    intensity: 1.0,
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
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.7,
                    color: (255, 100, 0), // Orange/Fire
                },
                SeismicSource {
                    intensity: 0.5,
                    radius: 3.0,
                },
                NoiseSource {
                    radius: 6.0,
                    intensity: 0.9,
                },
                PowerConsumer {
                    demand: 2.0,
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::StoneMason | BuildingType::Weaver | BuildingType::Tailor => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Refinery => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 20.0, // Slower process
                },
                LightSource {
                    is_outdoor: true,
                    radius: 6.0,
                    intensity: 0.8,
                    color: (100, 200, 255), // Chemical blue
                },
                SeismicSource {
                    intensity: 0.8,
                    radius: 6.0,
                },
                NoiseSource {
                    radius: 10.0,
                    intensity: 1.0,
                },
                ShiftSchedule::default(),
            ));
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
                MachineSpirit::default(),
                LightSource {
                    is_outdoor: true,
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
        _ => {}
    }
}

fn configure_production(entity: &mut EntityWorldMut, building_type: BuildingType) {
    configure_farm_buildings(entity, building_type);
    configure_refining_buildings(entity, building_type);
}

fn configure_storage(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Stockpile => {
            entity.insert(Stockpile::default());
        }
        BuildingType::Landfill => {
            entity.insert(Stockpile {
                waste_bonus: 100.0,
                food_bonus: 0.0,
                wood_bonus: 0.0,
                stone_bonus: 0.0,
            });
        }
        _ => {}
    }
}

fn configure_civic(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Tavern => {
            entity.insert((
                Tavern::default(),
                LightSource {
                    is_outdoor: true,
                    radius: 8.0,
                    intensity: 0.8,
                    color: (255, 140, 0), // Orange
                },
            ));
        }
        BuildingType::Library => {
            entity.insert((
                Library,
                LightSource {
                    is_outdoor: true,
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
        BuildingType::Hospital => {
            entity.insert((
                crate::layer1::medical::Hospital::default(),
                LightSource {
                    is_outdoor: true,
                    radius: 6.0,
                    intensity: 0.7,
                    color: (255, 255, 255), // Pure White
                },
                PowerConsumer {
                    demand: 5.0,
                    active: false,
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
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.6,
                    color: (220, 220, 100), // Yellowish
                },
            ));
        }
        BuildingType::CryoPod => {
            entity.insert((
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 2.0,
                    intensity: 0.4,
                    color: (0, 0, 255), // Deep Blue
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Shower => {
            // Has power consumer for water heater? Spec says "Use Water", not power.
            // But usually showers have lights or pumps.
            // I'll leave it basic for now.
            // Spec says: "Builder: Should Showers require Power? For now, no (gravity fed)."
        }
        _ => {}
    }
}

fn configure_infrastructure(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Window => {
            entity.insert((
                crate::layer1::window::Window {
                    range: 10,
                    direction: Direction::South, // Default view direction
                    ..Default::default()
                },
                // Explicitly add BeautySource (initialized to 0) so the window system can update it.
                // Normally spawn_building skips this if beauty_value is 0.
                BeautySource {
                    value: 0.0,
                    radius: 0.0,
                },
            ));
        }
        BuildingType::Gate => {
            entity.insert((
                crate::layer1::defense::Gate::default(),
                DoorControl::default(),
                AccessControl::default(),
            ));
        }
        BuildingType::Well => {
            entity.insert(WaterSource {
                range: 5,
                amount: MAX_HYDRATION,
            });
        }
        BuildingType::ConveyorBelt => {
            entity.insert((
                crate::layer1::logistics::ConveyorBelt {
                    direction: crate::layer1::building::Direction::East,
                    speed: 1.0,
                },
                PowerConsumer {
                    demand: 1.0,
                    active: false,
                },
            ));
        }
        BuildingType::Hopper => {
            entity.insert((
                crate::layer1::logistics::Hopper,
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
            ));
        }
        BuildingType::Airlock => {
            entity.insert((DoorControl::default(), AccessControl::default()));
        }
        _ => {}
    }
}

fn configure_power_generation(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Generator => {
            entity.insert((
                PowerSource {
                    output: 10.0,
                    ..Default::default()
                },
                FuelConsumer { amount: 1.0 },
                SeismicSource {
                    intensity: 1.0,
                    radius: 5.0,
                },
                NoiseSource {
                    radius: 8.0,
                    intensity: 0.8,
                },
            ));
        }
        BuildingType::SolarPanel => {
            entity.insert((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                SolarPower { base_output: 10.0 },
            ));
        }
        BuildingType::AncientReactor => {
            entity.insert((
                PowerSource {
                    output: 50.0,
                    ..Default::default()
                }, // Massive power
                AncientStructure,
                MachineSpirit::default(),
                LightSource {
                    is_outdoor: true,
                    radius: 8.0,
                    intensity: 1.0,
                    color: (255, 215, 0), // Gold
                },
                SeismicSource {
                    intensity: 3.0,
                    radius: 10.0,
                },
                NoiseSource {
                    radius: 12.0,
                    intensity: 1.0,
                },
            ));
            // Set high HP
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 1000.0;
                structure.current_hp = 1000.0;
            }
        }
        BuildingType::AuroralCollector => {
            entity.insert((
                PowerSource {
                    output: 0.0,
                    active: true,
                },
                crate::layer1::lighting::LightSource {
                    is_outdoor: true,
                    radius: 6.0,
                    intensity: 0.0,
                    color: (0, 255, 255), // Cyan
                },
            ));
        }
        _ => {}
    }
}

fn configure_power_infrastructure(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::PowerPole => {
            entity.insert(Conduit);
        }
        BuildingType::Battery => {
            entity.insert((
                crate::layer1::energy::Battery {
                    capacity: 100.0,
                    charge: 0.0,
                    max_throughput: 10.0,
                },
                Conduit,
            ));
        }
        _ => {}
    }
}

fn configure_power_consumption(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Heater => {
            entity.insert((
                PowerConsumer {
                    demand: 5.0,
                    active: true, // Typically on, logic will toggle if needed
                },
                crate::layer1::lighting::LightSource {
                    is_outdoor: true,
                    radius: 3.0,
                    intensity: 0.5,
                    color: (255, 100, 50), // Warm Orange
                },
            ));
        }
        BuildingType::AtmosphericProcessor => {
            entity.insert((
                PowerConsumer {
                    demand: 500.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 10.0,
                    intensity: 1.0,
                    color: (0, 255, 100), // Green
                },
                NoiseSource {
                    radius: 15.0,
                    intensity: 1.0,
                },
                SeismicSource {
                    intensity: 2.0,
                    radius: 8.0,
                },
            ));
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 2000.0;
                structure.current_hp = 2000.0;
            }
        }
        _ => {}
    }
}

fn configure_power(entity: &mut EntityWorldMut, building_type: BuildingType) {
    configure_power_generation(entity, building_type);
    configure_power_infrastructure(entity, building_type);
    configure_power_consumption(entity, building_type);
}

fn configure_science_buildings(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Observatory => {
            entity.insert((
                crate::layer1::observatory::Observatory { efficiency: 100.0 },
                crate::layer1::tech::Library, // Generates research implicitly via logic
                LightSource {
                    is_outdoor: true,
                    radius: 6.0,
                    intensity: 0.6,
                    color: (135, 206, 235), // Sky Blue
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::GeneBank => {
            entity.insert((
                crate::layer1::gene_bank::GeneBank::default(),
                PowerConsumer {
                    demand: 15.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.7,
                    color: (0, 255, 200), // Cyan/Green
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::ServerBank => {
            entity.insert((
                DataStorage { capacity: 50.0 },
                PowerConsumer {
                    demand: 10.0,
                    active: false, // Wait for power grid to activate
                },
                crate::layer1::lighting::LightSource {
                    is_outdoor: true,
                    radius: 2.0,
                    intensity: 0.4,
                    color: (0, 255, 100), // Data Green
                },
            ));
        }
        _ => {}
    }
}

fn configure_specialized_tech(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::TrashCannon => {
            entity.insert((
                crate::layer1::turret::Turret {
                    attack: crate::layer1::combat::AttackProperties {
                        damage: 15.0,
                        range: 7.0,
                        cooldown: 30,
                        accuracy: 0.9,
                    },
                    ammo_cost: 1.0,
                    ammo_type: crate::layer1::resources::ResourceType::Waste,
                },
                crate::layer1::combat::CombatState::default(),
                SeismicSource {
                    intensity: 2.0,
                    radius: 4.0,
                },
                NoiseSource {
                    radius: 10.0,
                    intensity: 1.0,
                },
            ));
        }
        BuildingType::LifeSupport => {
            entity.insert((
                PowerConsumer {
                    demand: 10.0,
                    active: true, // Always on if possible
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.6,
                    color: (200, 255, 255), // Cyan-ish
                },
            ));
        }
        BuildingType::CommandCenter => {
            entity.insert((
                PowerConsumer {
                    demand: 50.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.8,
                    color: (0, 0, 255), // Blue
                },
            ));
            // High HP
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 500.0;
                structure.current_hp = 500.0;
            }
        }
        BuildingType::AICore => {
            entity.insert((
                AICore::default(),
                PowerConsumer {
                    demand: 20.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.8,
                    color: (255, 0, 255), // Magenta/Purple
                },
            ));
        }
        BuildingType::DroneHub => {
            entity.insert((
                DroneHub,
                PowerConsumer {
                    demand: 10.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 3.0,
                    intensity: 0.6,
                    color: (0, 255, 255), // Cyan
                },
                NoiseSource {
                    radius: 5.0,
                    intensity: 0.5,
                },
            ));
        }
        _ => {}
    }
}

fn configure_futuristic_tech(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::CloneVat => {
            entity.insert((
                crate::layer1::clone_vat::CloneVat::default(),
                PowerConsumer {
                    demand: 20.0, // High power demand
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.6,
                    color: (0, 255, 100), // Greenish bio-light
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::HypnoPod => {
            entity.insert((
                crate::layer1::tech::hypno_learning::HypnoPod::default(),
                crate::layer1::housing::Housing {
                    capacity: 1,
                    residents: Vec::new(),
                },
                PowerConsumer {
                    demand: 15.0, // High power demand
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::HoloProjector => {
            entity.insert((
                crate::layer1::hologram::HoloProjector {
                    active_beauty: 50.0,
                    radius: 8.0,
                    is_active: false,
                },
                PowerConsumer {
                    demand: 10.0,
                    active: false,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.8,
                    color: (200, 200, 255), // Holographic Blue
                },
                // BeautySource added automatically by spawn_building based on BuildingType::beauty_value()
                // But HoloProjector toggles it.
                // spawn_building adds BeautySource { value: 50.0, radius: 8.0 } because beauty_value() returns 50.0.
                // We need to ensure it starts effectively "off" or let the system handle it.
                // The update_holograms_system will see is_active=false and set BeautySource.value=0.0 on first run if unpowered.
                // If powered, it sets it to active_beauty.
                ShiftSchedule::default(),
                // Explicitly initialize BeautySource to 0.0 so it starts off (overriding spawn_building default)
                BeautySource {
                    value: 0.0,
                    radius: 8.0,
                },
            ));
        }
        _ => {}
    }
}

fn configure_tech(entity: &mut EntityWorldMut, building_type: BuildingType) {
    configure_science_buildings(entity, building_type);
    configure_specialized_tech(entity, building_type);
    configure_futuristic_tech(entity, building_type);
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
fn check_tech_requirements(world: &mut World, building_type: BuildingType) -> bool {
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
    true
}

fn deduct_building_cost(
    world: &mut World,
    building_type: BuildingType,
    material: MaterialType,
) -> bool {
    let mut cost = building_type.cost(material);

    // Apply Scrapcode (Spec 178)
    if let Some(scrapcode) = world.get_resource::<crate::layer1::scrapcode::Scrapcode>() {
        if scrapcode.active {
            cost = cost * scrapcode.severity;
        }
    }

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
    true
}

fn apply_post_placement_effects(
    world: &mut World,
    entity: Entity,
    x: i32,
    y: i32,
    building_type: BuildingType,
) {
    // Vacuum Welding (Spec 185)
    if world
        .get_resource::<crate::layer1::pressure::PressureGrid>()
        .is_some_and(|p| p.get(x, y) < 0.1)
    {
        world.entity_mut(entity).insert(VacuumWelded);
        // Apply HP Bonus
        if let Some(mut structure) = world.get_mut::<crate::layer1::structure::Structure>(entity) {
            structure.max_hp *= 2.0;
            structure.current_hp *= 2.0;
        }
    }

    // Mark tile occupied
    world.resource_mut::<OccupiedTiles>().0.insert((x, y));

    // Ludwig: Juice - Add thud and dust
    if let Some(mut shake) = world.get_resource_mut::<crate::layer1::map::ScreenShake>() {
        shake.trigger(0.3);
    }
    crate::layer1::particles::spawn_particle(
        world,
        GridPosition { x, y },
        '*',
        ratatui::style::Color::White,
        10,
    );

    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add(format!("Construction started: {}", building_type.label()));
    }

    world.send_event(crate::layer1::events::BuildingCompletedEvent { entity });
}

pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
    // Check for Grave before validation
    let mut grave_entity = None;
    if let Some(map) = world.get_resource::<BuildingMap>() {
        if let Some(&entity) = map.0.get(&(x, y)) {
            if world.get::<crate::layer1::funeral::Grave>(entity).is_some() {
                grave_entity = Some(entity);
            }
        }
    }

    if let Err(e) = validate_building_placement(world, x, y) {
        let allow_override = e == PlacementError::Occupied && grave_entity.is_some();
        if !allow_override {
            handle_placement_error(world, e);
            return false;
        }
    }

    // Check Tech requirements
    if !check_tech_requirements(world, building_type) {
        return false;
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
    if !deduct_building_cost(world, building_type, material) {
        return false;
    }

    // --- All validation passed, commit to placing the building ---

    // If we are overwriting a grave, handle the sacrilege and destruction now
    if let Some(ge) = grave_entity {
        world.send_event(crate::layer1::ancestral_graves::SacrilegeEvent {
            pos: GridPosition { x, y },
        });
        // Remove grave synchronously
        world.despawn(ge);
    }

    // Spawn building
    let entity = spawn_building(world, x, y, building_type, material);

    apply_post_placement_effects(world, entity, x, y, building_type);

    true
}

/// Building component indicating it was constructed in a vacuum.
///
/// Vacuum Welded buildings:
/// - Have +100% Max HP.
/// - Cannot be Repaired or Demolished.
/// - Must be Destroyed (yielding 0 resources).
#[derive(Component, Default)]
pub struct VacuumWelded;

#[cfg(test)]
mod tests;
