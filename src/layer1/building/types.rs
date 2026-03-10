
use crate::layer1::resources::ColonyResources;
use crate::layer1::tech::Tech;
use bevy_ecs::prelude::*;

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
            Self::Statue => crate::layer1::beauty::STATUE_BEAUTY,
            Self::Landfill => -10.0,
            Self::Recycler => -5.0, // Grim machinery
            Self::Grave => -2.0,    // Graves are slightly spooky
            Self::FlowerBed => crate::layer1::beauty::FLOWER_BED_BEAUTY,
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
/// Component defining the material of a building.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Material(pub MaterialType);
