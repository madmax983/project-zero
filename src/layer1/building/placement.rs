use crate::layer1::access_control::AccessControl;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::building::building_components::*;
use crate::layer1::building::types::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::tech::TechState;
use crate::layer1::map::GridPosition;
use crate::layer1::inventory::Inventory;
use crate::layer1::permit::PermitRequired;
use crate::layer1::fire::Flammable;
use crate::layer1::beauty::BeautySource;
use crate::layer1::admin::AdminConsumer;
use crate::layer1::atmosphere::CorrosionResistant;
use crate::layer1::heirloom::AncientStructure;
use crate::layer1::rituals::MachineSpirit;
use crate::layer1::prototyping::{BuildingMastery, Prototype};
use crate::layer1::housing::Housing;
use crate::layer1::farm::Farm;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::tech::{Library, DataStorage};
use crate::layer1::trade::TradeDepot;
use crate::layer1::drone::DroneHub;
use crate::layer1::energy::{Conduit, FuelConsumer, PowerConsumer, PowerSource};
use crate::layer1::lighting::LightSource;
use crate::layer1::seismic::SeismicSource;
use crate::layer1::solar::SolarPower;
use crate::layer1::admin::{AdminProvider, Office};
use crate::layer1::ai_core::AICore;
use crate::layer1::control::DoorControl;
use crate::layer1::items::ItemType;
use crate::layer1::resources::RefiningProgress;
use crate::layer1::water::{WaterSource, MAX_HYDRATION};
use crate::layer1::acoustic::NoiseSource;
use crate::layer1::social::Tavern;
use bevy_ecs::prelude::*;
use crate::shared::log::MessageLog;
use rand::seq::SliceRandom;
// use super::*;

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
        BuildingType::Office => {
            entity.insert((
                // Office provides admin
                AdminProvider { amount: 10.0 },
                Office::default(),
                // Office typically operates during the day
                ShiftSchedule::default(),
            ));
        }
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
        BuildingType::Nanoforge => {
            entity.insert((crate::layer1::nanite_fabrication::Nanoforge {
                active_recipe: None,
                breach_risk: 0.01,
            },));
        }
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
        BuildingType::Office
        | BuildingType::Tavern
        | BuildingType::Library
        | BuildingType::FlowerBed
        | BuildingType::Statue
        | BuildingType::Hospital
        | BuildingType::Grave
        | BuildingType::TradeDepot
        | BuildingType::Shower
        | BuildingType::Recycler
        | BuildingType::BulletinBoard => configure_civic(&mut entity, building_type),
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
        | BuildingType::HoloProjector
        | BuildingType::Nanoforge => configure_tech(&mut entity, building_type),
        BuildingType::PersonalShed
        | BuildingType::PersonalGarden
        | BuildingType::PersonalShrine => {
            // Logic handled by components added in system
        }
    }

    entity.id()
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
