with open("src/layer1/building/mod.rs", "r") as f:
    mod_c = f.read()

# Instead of `//! # Entities`, let's just make sure we only use `//!` at the top of the file!
# Inner docs can only be at the top level of a module.
# Let's just fix it by replacing them with `///` or `//`
mod_c = mod_c.replace("//! # Entities", "// # Entities")
mod_c = mod_c.replace("\n//!\n", "\n//\n")
mod_c = mod_c.replace("//! A built structure", "// A built structure")
mod_c = mod_c.replace("//! *   [`Building`]", "// *   [`Building`]")
mod_c = mod_c.replace("//! *   [`GridPosition`]", "// *   [`GridPosition`]")
mod_c = mod_c.replace("//! *   [`crate::layer1::structure::Structure`]", "// *   [`crate::layer1::structure::Structure`]")
mod_c = mod_c.replace("//! *   Specific Logic Components:", "// *   Specific Logic Components:")
mod_c = mod_c.replace("//! # Core Systems", "// # Core Systems")
mod_c = mod_c.replace("//! *   **Placement:**", "// *   **Placement:**")
mod_c = mod_c.replace("//! *   **Cost:**", "// *   **Cost:**")
mod_c = mod_c.replace("//! *   **Tech:**", "// *   **Tech:**")
mod_c = mod_c.replace("//! *   **Obstacles:**", "// *   **Obstacles:**")
mod_c = mod_c.replace("//! Buildings are the primary structures", "// Buildings are the primary structures")
mod_c = mod_c.replace("//! production, defense", "// production, defense")

# Remove unused imports from mod.rs
mod_c = mod_c.replace("use crate::layer1::access_control::AccessControl;", "")
mod_c = mod_c.replace("use crate::layer1::admin::{AdminConsumer, AdminProvider, Office};", "")
mod_c = mod_c.replace("use crate::layer1::ai_core::AICore;", "")
mod_c = mod_c.replace("use crate::layer1::atmosphere::CorrosionResistant;", "")
mod_c = mod_c.replace("use crate::layer1::control::DoorControl;", "")
mod_c = mod_c.replace("use crate::layer1::drone::DroneHub;", "")
mod_c = mod_c.replace("use crate::layer1::energy::{Conduit, FuelConsumer, PowerConsumer, PowerSource};", "")
mod_c = mod_c.replace("use crate::layer1::heirloom::AncientStructure;", "")
mod_c = mod_c.replace("use crate::layer1::inventory::Inventory;", "")
mod_c = mod_c.replace("use crate::layer1::items::ItemType;", "")
mod_c = mod_c.replace("use crate::layer1::lighting::LightSource;", "")
mod_c = mod_c.replace("use crate::layer1::permit::PermitRequired;", "")
mod_c = mod_c.replace("use crate::layer1::prototyping::{BuildingMastery, Prototype};", "")
mod_c = mod_c.replace("use crate::layer1::resources::{ColonyResources, RefiningProgress};", "use crate::layer1::resources::ColonyResources;")
mod_c = mod_c.replace("use crate::layer1::rituals::MachineSpirit;", "")
mod_c = mod_c.replace("use crate::layer1::seismic::SeismicSource;", "")
mod_c = mod_c.replace("use crate::layer1::solar::SolarPower;", "")
mod_c = mod_c.replace("use crate::layer1::tech::{DataStorage, Library, Tech, TechState};", "use crate::layer1::tech::{Tech, TechState};")
mod_c = mod_c.replace("use crate::layer1::trade::TradeDepot;", "")
mod_c = mod_c.replace("use crate::layer1::water::{WaterSource, MAX_HYDRATION};", "")
mod_c = mod_c.replace("use super::acoustic::NoiseSource;", "")
mod_c = mod_c.replace("use super::beauty::BeautySource;", "")
mod_c = mod_c.replace("use super::farm::Farm;", "")
mod_c = mod_c.replace("use super::fire::Flammable;", "")
mod_c = mod_c.replace("use super::housing::Housing;", "")
mod_c = mod_c.replace("use super::social::Tavern;", "")
mod_c = mod_c.replace("use super::stockpile::Stockpile;", "")
mod_c = mod_c.replace("use bevy_ecs::world::EntityWorldMut;", "")
mod_c = mod_c.replace("use rand::seq::SliceRandom;", "")


with open("src/layer1/building/mod.rs", "w") as f:
    f.write(mod_c)

with open("src/layer1/building/configuration.rs", "r") as f:
    conf_c = f.read()
conf_c = conf_c.replace("use crate::layer1::resources::{ColonyResources, RefiningProgress};", "use crate::layer1::resources::RefiningProgress;")
conf_c = conf_c.replace("use crate::layer1::tech::{DataStorage, Library, Tech, TechState};", "use crate::layer1::tech::{DataStorage, Library};")
conf_c = conf_c.replace("use crate::layer1::terrain::{TerrainGrid, TerrainType};\n", "")
with open("src/layer1/building/configuration.rs", "w") as f:
    f.write(conf_c)
