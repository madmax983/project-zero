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

pub mod components;
pub mod placement;
/// Direction for buildings (e.g., Conveyor Belts).
pub mod types;

pub use components::{
    apply_post_placement_effects, can_place_building, check_tech_requirements,
    deduct_building_cost, handle_placement_error, spawn_building, spawn_building_with_material,
    update_building_map_system, validate_building_placement, BuildMode, Building, BuildingMap,
    OccupiedTiles, PlacementError, ShiftSchedule, VacuumWelded,
};
pub use placement::try_place_building;
pub use types::{BuildingType, Category, Direction, Material, MaterialType, Tier};

#[cfg(test)]
mod tests;
