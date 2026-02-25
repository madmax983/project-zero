//! Resource management and mining mechanics.
//!
//! This module defines the colony's economic backbone: `ColonyResources` and the
//! mechanisms to extract them from the environment (e.g., `mine_rock`).
//!
//! # Key Concepts
//!
//! * **ColonyResources**: The global stockpile of Food, Wood, and Stone.
//! * **Mining**: A multi-tick process tracked by `MiningProgress` that converts
//!   terrain (Rock -> Dirt) and yields resources (Stone).
//!
//! # The Mining Loop
//!
//! 1. Player designates a tile (see `crate::layer1::designation`).
//! 2. A pop is assigned the job (see `crate::layer1::pop`).
//! 3. The pop works on the tile, calling `mine_rock`.
//! 4. `MiningProgress` accumulates.
//! 5. Upon completion, the tile changes and resources are awarded.

use crate::layer1::GridPosition;
use crate::layer1::science::{Anomaly, AnomalyType, ScanProgress};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::ops::Mul;

/// Types of resources in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Consumed by pops to reduce hunger.
    Food,
    /// Raw wood from trees.
    Wood,
    /// Raw stone from rocks.
    Stone,
    /// Raw ore from mining rocks.
    Ore,
    /// Refined metal from ore.
    Metal,
    /// Refined wood from lumber mill.
    Planks,
    /// Refined stone from mason.
    Blocks,
    /// Industrial waste (pollution).
    Waste,
    /// Rations for survival (high nutrition).
    Rations,
    /// Fuel for reactors and ships.
    Fuel,
    /// Alcohol (consumable, potentially contraband).
    Alcohol,
    /// High-tech scrap from orbital debris.
    Scrap,
    /// Standard tools (Pickaxe, Axe, Hammer).
    Tools,
    /// Permit required to construct advanced buildings.
    BuildingPermit,
}

/// A physical resource item in the world (dropped on the ground).
#[derive(Component, Debug, Clone, Copy)]
pub struct ResourceItem {
    /// The type of resource.
    pub resource_type: ResourceType,
    /// The quantity of the resource.
    pub amount: f32,
}

/// Component indicating a Pop is carrying a resource.
#[derive(Component, Debug, Clone, Copy)]
pub struct Carrying {
    /// The type of resource being carried.
    pub resource_type: ResourceType,
    /// The quantity of the resource.
    pub amount: f32,
}

/// Tracks the resources available to the colony.
///
/// This resource serves as the "bank" for the simulation.
///
/// # Examples
///
/// Basic usage:
/// ```
/// use scale::layer1::resources::ColonyResources;
///
/// let mut resources = ColonyResources::default();
/// resources.food += 10.0;
/// assert_eq!(resources.food, 20.0); // default starts with 10.0
/// ```
///
/// # The Hero's Journey: Building a Wall
///
/// Suppose you want to build a wall that costs 5 Stone.
/// Use [`try_deduct`](ColonyResources::try_deduct) to ensure you don't overspend.
///
/// ```
/// use scale::layer1::resources::ColonyResources;
///
/// let mut stockpile = ColonyResources::default();
/// stockpile.stone = 4.0; // Not enough!
///
/// let wall_cost = ColonyResources {
///     stone: 5.0,
///     ..ColonyResources::zeroed()
/// };
///
/// if stockpile.try_deduct(&wall_cost) {
///     println!("Wall built!");
/// } else {
///     println!("Not enough stone, my lord.");
/// }
/// ```
#[derive(Resource, Debug, Clone)]
pub struct ColonyResources {
    /// Total food available in the colony.
    pub food: f32,
    /// Total wheat available.
    pub wheat: f32,
    /// Total potato available.
    pub potato: f32,
    /// Total rice available.
    pub rice: f32,
    /// Total wood available in the colony.
    pub wood: f32,
    /// Total stone available in the colony.
    pub stone: f32,
    /// Total planks available in the colony.
    pub planks: f32,
    /// Total blocks available in the colony.
    pub blocks: f32,
    /// Total ore available in the colony.
    pub ore: f32,
    /// Total metal available in the colony.
    pub metal: f32,
    /// Total tools available in the colony.
    pub tools: f32,
    /// Total knowledge available in the colony.
    pub knowledge: f32,
    /// Total fiber available in the colony.
    pub fiber: f32,
    /// Total cloth available in the colony.
    pub cloth: f32,
    /// Total clothing available in the colony.
    pub clothing: f32,
    /// Total waste accumulated in the colony (must be hauled to landfill).
    pub waste: f32,
    /// Total rations available in the colony.
    pub rations: f32,
    /// Total fuel available in the colony.
    pub fuel: f32,
    /// Total water available in the colony.
    pub water: f32,
    /// Total alcohol available in the colony.
    pub alcohol: f32,
    /// Total scrap available in the colony.
    pub scrap: f32,
    /// Total building permits available in the colony.
    pub building_permits: f32,
    /// Maximum food capacity.
    pub max_food: f32,
    /// Maximum wood capacity.
    pub max_wood: f32,
    /// Maximum stone capacity.
    pub max_stone: f32,
    /// Maximum planks capacity.
    pub max_planks: f32,
    /// Maximum blocks capacity.
    pub max_blocks: f32,
    /// Maximum ore capacity.
    pub max_ore: f32,
    /// Maximum metal capacity.
    pub max_metal: f32,
    /// Maximum tools capacity.
    pub max_tools: f32,
    /// Maximum knowledge capacity.
    pub max_knowledge: f32,
    /// Maximum fiber capacity.
    pub max_fiber: f32,
    /// Maximum cloth capacity.
    pub max_cloth: f32,
    /// Maximum clothing capacity.
    pub max_clothing: f32,
    /// Maximum waste capacity.
    pub max_waste: f32,
    /// Maximum rations capacity.
    pub max_rations: f32,
    /// Maximum fuel capacity.
    pub max_fuel: f32,
    /// Maximum water capacity.
    pub max_water: f32,
    /// Maximum alcohol capacity.
    pub max_alcohol: f32,
    /// Maximum scrap capacity.
    pub max_scrap: f32,
    /// Maximum building permits capacity (usually infinite or high).
    pub max_building_permits: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            food: 10.0,
            wheat: 0.0,
            potato: 0.0,
            rice: 0.0,
            wood: 15.0,
            stone: 5.0,
            planks: 0.0,
            blocks: 0.0,
            ore: 0.0,
            metal: 0.0,
            tools: 2.0,
            knowledge: 0.0,
            fiber: 0.0,
            cloth: 0.0,
            clothing: 0.0,
            waste: 0.0,
            rations: 0.0,
            fuel: 0.0,
            alcohol: 0.0,
            scrap: 0.0,
            max_food: 50.0,
            max_wood: 50.0,
            max_stone: 20.0,
            max_planks: 50.0,
            max_blocks: 20.0,
            max_ore: 20.0,
            max_metal: 20.0,
            max_tools: 50.0,
            max_knowledge: 100.0, // Default for tests
            max_fiber: 50.0,
            max_cloth: 50.0,
            max_clothing: 50.0,
            max_waste: 0.0, // Defaults to 0, requires Landfill
            max_rations: 50.0,
            max_fuel: 20.0,
            water: 0.0,
            max_water: 50.0,
            max_alcohol: 50.0,
            max_scrap: 20.0,
            building_permits: 0.0,
            max_building_permits: 100.0,
        }
    }
}

impl Mul<f32> for ColonyResources {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            food: (self.food * rhs).ceil(),
            wheat: (self.wheat * rhs).ceil(),
            potato: (self.potato * rhs).ceil(),
            rice: (self.rice * rhs).ceil(),
            wood: (self.wood * rhs).ceil(),
            stone: (self.stone * rhs).ceil(),
            planks: (self.planks * rhs).ceil(),
            blocks: (self.blocks * rhs).ceil(),
            ore: (self.ore * rhs).ceil(),
            metal: (self.metal * rhs).ceil(),
            tools: (self.tools * rhs).ceil(),
            knowledge: (self.knowledge * rhs).ceil(),
            fiber: (self.fiber * rhs).ceil(),
            cloth: (self.cloth * rhs).ceil(),
            clothing: (self.clothing * rhs).ceil(),
            waste: (self.waste * rhs).ceil(),
            rations: (self.rations * rhs).ceil(),
            fuel: (self.fuel * rhs).ceil(),
            water: (self.water * rhs).ceil(),
            alcohol: (self.alcohol * rhs).ceil(),
            scrap: (self.scrap * rhs).ceil(),
            // Capacities should NOT change when multiplying cost
            max_food: self.max_food,
            max_wood: self.max_wood,
            max_stone: self.max_stone,
            max_planks: self.max_planks,
            max_blocks: self.max_blocks,
            max_ore: self.max_ore,
            max_metal: self.max_metal,
            max_tools: self.max_tools,
            max_knowledge: self.max_knowledge,
            max_fiber: self.max_fiber,
            max_cloth: self.max_cloth,
            max_clothing: self.max_clothing,
            max_waste: self.max_waste,
            max_rations: self.max_rations,
            max_fuel: self.max_fuel,
            max_water: self.max_water,
            max_alcohol: self.max_alcohol,
            max_scrap: self.max_scrap,
            building_permits: (self.building_permits * rhs).ceil(),
            max_building_permits: self.max_building_permits,
        }
    }
}

impl ColonyResources {
    /// Returns a `ColonyResources` with all values set to zero.
    ///
    /// Use this for cost structs and other contexts where you need a blank slate
    /// rather than the colony's starting resources.
    #[must_use]
    pub const fn zeroed() -> Self {
        Self {
            food: 0.0,
            wheat: 0.0,
            potato: 0.0,
            rice: 0.0,
            wood: 0.0,
            stone: 0.0,
            planks: 0.0,
            blocks: 0.0,
            ore: 0.0,
            metal: 0.0,
            tools: 0.0,
            knowledge: 0.0,
            fiber: 0.0,
            cloth: 0.0,
            clothing: 0.0,
            waste: 0.0,
            rations: 0.0,
            fuel: 0.0,
            alcohol: 0.0,
            scrap: 0.0,
            max_food: 0.0,
            max_wood: 0.0,
            max_stone: 0.0,
            max_planks: 0.0,
            max_blocks: 0.0,
            max_ore: 0.0,
            max_metal: 0.0,
            max_tools: 0.0,
            max_knowledge: 0.0,
            max_fiber: 0.0,
            max_cloth: 0.0,
            max_clothing: 0.0,
            max_waste: 0.0,
            max_rations: 0.0,
            max_fuel: 0.0,
            water: 0.0,
            max_water: 0.0,
            max_alcohol: 0.0,
            max_scrap: 0.0,
            building_permits: 0.0,
            max_building_permits: 0.0,
        }
    }

    /// Adds scrap, clamping to the maximum capacity.
    pub fn add_scrap(&mut self, amount: f32) {
        self.scrap = (self.scrap + amount).clamp(0.0, self.max_scrap);
    }

    /// Adds building permits, clamping to the maximum capacity.
    pub fn add_building_permits(&mut self, amount: f32) {
        self.building_permits =
            (self.building_permits + amount).clamp(0.0, self.max_building_permits);
    }

    /// Adds alcohol, clamping to the maximum capacity.
    pub fn add_alcohol(&mut self, amount: f32) {
        self.alcohol = (self.alcohol + amount).clamp(0.0, self.max_alcohol);
    }

    /// Adds water, clamping to the maximum capacity.
    pub fn add_water(&mut self, amount: f32) {
        self.water = (self.water + amount).clamp(0.0, self.max_water);
    }

    /// Adds rations, clamping to the maximum capacity.
    pub fn add_rations(&mut self, amount: f32) {
        self.rations = (self.rations + amount).clamp(0.0, self.max_rations);
    }

    /// Adds fuel, clamping to the maximum capacity.
    pub fn add_fuel(&mut self, amount: f32) {
        self.fuel = (self.fuel + amount).clamp(0.0, self.max_fuel);
    }

    /// Adds wood, clamping to the maximum capacity.
    pub fn add_wood(&mut self, amount: f32) {
        self.wood = (self.wood + amount).clamp(0.0, self.max_wood);
    }

    /// Adds stone, clamping to the maximum capacity.
    pub fn add_stone(&mut self, amount: f32) {
        self.stone = (self.stone + amount).clamp(0.0, self.max_stone);
    }

    /// Adds food, clamping to the maximum capacity.
    pub fn add_food(&mut self, amount: f32) {
        self.food = (self.food + amount).clamp(0.0, self.max_food);
    }

    /// Returns the total food available (food aggregate + rations).
    #[must_use]
    pub fn total_food(&self) -> f32 {
        self.food + self.rations
    }

    /// Adds planks, clamping to the maximum capacity.
    pub fn add_planks(&mut self, amount: f32) {
        self.planks = (self.planks + amount).clamp(0.0, self.max_planks);
    }

    /// Adds blocks, clamping to the maximum capacity.
    pub fn add_blocks(&mut self, amount: f32) {
        self.blocks = (self.blocks + amount).clamp(0.0, self.max_blocks);
    }

    /// Adds ore, clamping to the maximum capacity.
    pub fn add_ore(&mut self, amount: f32) {
        self.ore = (self.ore + amount).clamp(0.0, self.max_ore);
    }

    /// Adds metal, clamping to the maximum capacity.
    pub fn add_metal(&mut self, amount: f32) {
        self.metal = (self.metal + amount).clamp(0.0, self.max_metal);
    }

    /// Adds tools, clamping to the maximum capacity.
    pub fn add_tools(&mut self, amount: f32) {
        self.tools = (self.tools + amount).clamp(0.0, self.max_tools);
    }

    /// Adds knowledge, clamping to the maximum capacity.
    pub fn add_knowledge(&mut self, amount: f32) {
        self.knowledge = (self.knowledge + amount).clamp(0.0, self.max_knowledge);
    }

    /// Adds fiber, clamping to the maximum capacity.
    pub fn add_fiber(&mut self, amount: f32) {
        self.fiber = (self.fiber + amount).clamp(0.0, self.max_fiber);
    }

    /// Adds cloth, clamping to the maximum capacity.
    pub fn add_cloth(&mut self, amount: f32) {
        self.cloth = (self.cloth + amount).clamp(0.0, self.max_cloth);
    }

    /// Adds clothing, clamping to the maximum capacity.
    pub fn add_clothing(&mut self, amount: f32) {
        self.clothing = (self.clothing + amount).clamp(0.0, self.max_clothing);
    }

    /// Adds waste, clamping to the maximum capacity.
    pub fn add_waste(&mut self, amount: f32) {
        self.waste = (self.waste + amount).clamp(0.0, self.max_waste);
    }

    /// Checks if any resource value is negative.
    ///
    /// Used for validation to prevent exploit vectors where negative costs
    /// effectively add resources.
    #[must_use]
    pub fn has_negative(&self) -> bool {
        self.food < 0.0
            || self.wheat < 0.0
            || self.potato < 0.0
            || self.rice < 0.0
            || self.wood < 0.0
            || self.stone < 0.0
            || self.planks < 0.0
            || self.blocks < 0.0
            || self.ore < 0.0
            || self.metal < 0.0
            || self.tools < 0.0
            || self.knowledge < 0.0
            || self.fiber < 0.0
            || self.cloth < 0.0
            || self.clothing < 0.0
            || self.rations < 0.0
            || self.fuel < 0.0
            || self.water < 0.0
            || self.alcohol < 0.0
            || self.scrap < 0.0
            || self.building_permits < 0.0
    }

    /// Checks if the colony can afford the given cost.
    ///
    /// Also validates that the cost is non-negative to prevent exploits.
    ///
    /// # Parameters
    ///
    /// * `cost`: The resources required.
    ///
    /// # Returns
    ///
    /// True if all resources are sufficient and cost is valid.
    #[must_use]
    pub fn can_afford(&self, cost: &Self) -> bool {
        if cost.has_negative() {
            return false;
        }

        self.food >= cost.food
            && self.wheat >= cost.wheat
            && self.potato >= cost.potato
            && self.rice >= cost.rice
            && self.wood >= cost.wood
            && self.stone >= cost.stone
            && self.planks >= cost.planks
            && self.blocks >= cost.blocks
            && self.ore >= cost.ore
            && self.metal >= cost.metal
            && self.tools >= cost.tools
            && self.knowledge >= cost.knowledge
            && self.fiber >= cost.fiber
            && self.cloth >= cost.cloth
            && self.clothing >= cost.clothing
            && self.rations >= cost.rations
            && self.fuel >= cost.fuel
            && self.water >= cost.water
            && self.alcohol >= cost.alcohol
            && self.scrap >= cost.scrap
            && self.building_permits >= cost.building_permits
    }

    /// Deducts the given cost from the colony's resources.
    ///
    /// # Parameters
    ///
    /// * `cost`: The resources to deduct.
    fn deduct(&mut self, cost: &Self) {
        self.food -= cost.food;
        self.wheat -= cost.wheat;
        self.potato -= cost.potato;
        self.rice -= cost.rice;
        self.wood -= cost.wood;
        self.stone -= cost.stone;
        self.planks -= cost.planks;
        self.blocks -= cost.blocks;
        self.ore -= cost.ore;
        self.metal -= cost.metal;
        self.tools -= cost.tools;
        self.knowledge -= cost.knowledge;
        self.fiber -= cost.fiber;
        self.cloth -= cost.cloth;
        self.clothing -= cost.clothing;
        self.rations -= cost.rations;
        self.fuel -= cost.fuel;
        self.water -= cost.water;
        self.alcohol -= cost.alcohol;
        self.scrap -= cost.scrap;
        self.building_permits -= cost.building_permits;
    }

    /// Attempts to deduct the given cost from the colony's resources.
    ///
    /// Checks affordability first. If affordable, deducts and returns `true`.
    /// Otherwise, returns `false` and makes no changes.
    ///
    /// # Parameters
    ///
    /// * `cost`: The resources to deduct.
    pub fn try_deduct(&mut self, cost: &Self) -> bool {
        if self.can_afford(cost) {
            self.deduct(cost);
            true
        } else {
            false
        }
    }

    /// Consumes a specific amount of a single resource type.
    ///
    /// This is a convenience method for simple deductions. It clamps the deduction
    /// to the available amount (it will not go negative).
    ///
    /// # Parameters
    ///
    /// * `resource_type`: The type of resource to consume.
    /// * `amount`: The amount to consume.
    pub fn consume(&mut self, resource_type: ResourceType, amount: f32) {
        match resource_type {
            ResourceType::Food => self.food = (self.food - amount).max(0.0),
            ResourceType::Wood => self.wood = (self.wood - amount).max(0.0),
            ResourceType::Stone => self.stone = (self.stone - amount).max(0.0),
            ResourceType::Ore => self.ore = (self.ore - amount).max(0.0),
            ResourceType::Metal => self.metal = (self.metal - amount).max(0.0),
            ResourceType::Planks => self.planks = (self.planks - amount).max(0.0),
            ResourceType::Blocks => self.blocks = (self.blocks - amount).max(0.0),
            ResourceType::Tools => self.tools = (self.tools - amount).max(0.0),
            ResourceType::Scrap => self.scrap = (self.scrap - amount).max(0.0),
            ResourceType::Fuel => self.fuel = (self.fuel - amount).max(0.0),
            ResourceType::Rations => self.rations = (self.rations - amount).max(0.0),
            // ResourceType::Water not in enum
            ResourceType::Alcohol => self.alcohol = (self.alcohol - amount).max(0.0),
            ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),
            ResourceType::BuildingPermit => {
                self.building_permits = (self.building_permits - amount).max(0.0);
            }
        }
    }
}

/// Component tracking the progress of a mining designation.
///
/// Attached to entities that are being actively mined. The simulation uses this
/// to persist work across multiple ticks/frames.
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::MiningProgress;
///
/// let mut progress = MiningProgress { current: 50.0, max: 100.0 };
/// assert!(!progress.is_complete());
///
/// // Simulate work
/// progress.current += 60.0;
/// assert!(progress.is_complete());
/// ```
#[derive(Component, Debug)]
pub struct MiningProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the mining.
    pub max: f32,
}

/// Base maximum food capacity.
pub const BASE_MAX_FOOD: f32 = 50.0;
/// Base maximum wood capacity.
pub const BASE_MAX_WOOD: f32 = 50.0;
/// Base maximum stone capacity.
pub const BASE_MAX_STONE: f32 = 20.0;

impl MiningProgress {
    /// Returns true if the work is finished.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

/// Component tracking the progress of a forestry designation.
#[derive(Component, Debug, Default)]
pub struct ForestryProgress {
    /// Current amount of work done.
    pub current: f32,
    /// Total work required to complete the chopping.
    pub max: f32,
}

impl ForestryProgress {
    /// Returns true if the work is finished.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

/// Component tracking the progress of refining (e.g. at a Lumber Mill).
///
/// This component acts as a state container for the [`process_refining_system`](crate::layer1::refining::process_refining_system).
/// It persists the current work amount across ticks.
///
/// # State Logic
///
/// * **`current`**: Accumulates as workers apply effort (scaled by efficiency).
/// * **`max`**: The threshold to complete one batch.
///
/// When `current >= max`, the system triggers completion logic (resource swap, XP gain)
/// and typically resets `current` to 0.0 or despawns the job depending on context.
#[derive(Component, Debug, Default)]
pub struct RefiningProgress {
    /// Current work done.
    pub current: f32,
    /// Work required to finish one batch.
    pub max: f32,
}

impl RefiningProgress {
    /// Returns true if the work is finished.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

/// Applies work to a mining designation.
///
/// This function is the core of the mining mechanic. It advances the `MiningProgress`
/// of a specific designation. If the work completes the task, it:
/// 1. Despawns the designation.
/// 2. Changes the terrain from `Rock` to `Dirt`.
/// 3. Spawns `Stone` as a `ResourceItem` (must be hauled).
///
/// # Parameters
///
/// * `world`: Mutable access to the ECS world (needed to modify terrain and resources).
/// * `designation_entity`: The entity ID of the designation being worked on.
/// * `work_amount`: How much progress to add (usually based on worker skill/speed).
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::{mine_rock, ColonyResources, MiningProgress, ResourceItem, ResourceType};
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use scale::layer1::GridPosition;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
///
/// // 1. Setup World
/// let mut tiles = vec![TerrainType::Grass; 100];
/// tiles[0] = TerrainType::Rock; // Target is rock
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(ColonyResources::default());
/// // RoofGrid is required for structural stability checks
/// world.insert_resource(scale::layer1::structural_integrity::RoofGrid::new(10, 10));
///
/// // 2. Create Designation
/// let designation = world.spawn((
///     GridPosition { x: 0, y: 0 },
///     MiningProgress { current: 0.0, max: 10.0 }
/// )).id();
///
/// // 3. Work until done
/// mine_rock(&mut world, designation, 10.0);
///
/// // 4. Verify Result
/// // The resource is spawned as an item on the ground, not added directly to stocks.
/// let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
/// assert!(!items.is_empty());
/// assert_eq!(items[0].resource_type, ResourceType::Stone);
/// ```
#[allow(clippy::cast_sign_loss)]
pub fn mine_rock(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify terrain
    let (pos, is_rock) = {
        let pos = if let Some(p) = world.get::<GridPosition>(designation_entity) {
            *p
        } else {
            return;
        };

        if pos.x < 0 || pos.y < 0 {
            return;
        }

        let terrain = world.resource::<TerrainGrid>();
        // Safe to cast because we checked for negative above
        let is_rock = terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Rock);
        (pos, is_rock)
    };

    if !is_rock {
        return;
    }

    // 2. Update progress
    let completed = if let Some(mut progress) = world.get_mut::<MiningProgress>(designation_entity)
    {
        progress.current += work_amount;
        progress.current >= progress.max
    } else {
        false
    };

    // 3. Handle completion
    if completed {
        // Change terrain
        // Scope the borrow of terrain so we can use world later
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            // Check bounds again? Technically redundant if terrain didn't shrink, but safe.
            // Also we checked < 0 earlier.
            let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
            if idx < terrain.tiles.len() {
                terrain.tiles[idx] = TerrainType::Dirt;
            }
        }

        // Spawn visual item on the ground (MUST BE HAULED)
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Stone,
                amount: 1.0,
            },
            pos,
        ));

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("Mined Stone (Needs Hauling)");
        }

        // Purity Logic: Ore (High Purity) vs Waste (Low Purity)
        let purity = world
            .get_resource::<crate::layer1::purity::PurityMap>()
            .map_or(0.2, |map| map.get(pos.x, pos.y));

        let mut rng = rand::thread_rng();

        // Ore Check: Probability = Purity
        if rng.gen_bool(f64::from(purity)) {
            world.spawn((
                ResourceItem {
                    resource_type: ResourceType::Ore,
                    amount: 1.0,
                },
                pos,
            ));
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add("Mined Ore (Needs Hauling)");
            }
        }

        // Waste Check: Probability = 1.0 - Purity
        // This is independent, so mixed purity can yield both or neither.
        if rng.gen_bool(f64::from(1.0 - purity)) {
            world.spawn((
                ResourceItem {
                    resource_type: ResourceType::Waste,
                    amount: 1.0,
                },
                pos,
            ));
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add("Mined Waste (Needs Hauling)");
            }
        }

        // Probabilistic Anomaly Spawn (5%)
        try_spawn_anomaly(world, pos);

        // Remove designation
        world.despawn(designation_entity);

        // Check Stability
        if !crate::layer1::structural_integrity::check_stability(world, pos) {
            crate::layer1::structural_integrity::apply_collapse(world, pos);
        }
    }
}

/// Applies work to a forestry designation (chopping a tree).
///
/// Similar to `mine_rock`, but for trees.
///
/// # Parameters
///
/// * `world`: Mutable access to the ECS world.
/// * `designation_entity`: The entity ID of the designation.
/// * `work_amount`: How much progress to add.
#[allow(clippy::cast_sign_loss)]
pub fn chop_tree(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify terrain
    let (pos, is_tree) = {
        let pos = if let Some(p) = world.get::<GridPosition>(designation_entity) {
            *p
        } else {
            return;
        };

        if pos.x < 0 || pos.y < 0 {
            return;
        }

        let terrain = world.resource::<TerrainGrid>();
        let is_tree = terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Tree);
        (pos, is_tree)
    };

    if !is_tree {
        return;
    }

    // 2. Update progress
    let completed =
        if let Some(mut progress) = world.get_mut::<ForestryProgress>(designation_entity) {
            progress.current += work_amount;
            progress.current >= progress.max
        } else {
            false
        };

    // 3. Handle completion
    if completed {
        // Change terrain
        let mut terrain = world.resource_mut::<TerrainGrid>();
        let idx = (pos.y as usize) * terrain.width + (pos.x as usize);
        if idx < terrain.tiles.len() {
            terrain.tiles[idx] = TerrainType::Dirt;
        }

        // Spawn visual item on the ground (MUST BE HAULED)
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Wood,
                amount: 1.0,
            },
            pos,
        ));

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("Chopped Tree (Needs Hauling)");
        }

        // Remove designation
        world.despawn(designation_entity);
    }
}

/// Helper to ensure `MiningProgress` component exists and call `mine_rock`.
pub fn process_mining(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // Ensure MiningProgress exists
    if world.get::<MiningProgress>(designation_entity).is_none() {
        world
            .entity_mut(designation_entity)
            .insert(MiningProgress::default());
    }
    mine_rock(world, designation_entity, work_amount);
}

/// Helper to ensure `ForestryProgress` component exists and call `chop_tree`.
pub fn process_logging(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // Ensure ForestryProgress exists
    if world.get::<ForestryProgress>(designation_entity).is_none() {
        world
            .entity_mut(designation_entity)
            .insert(ForestryProgress {
                current: 0.0,
                max: 50.0,
            });
    }
    chop_tree(world, designation_entity, work_amount);
}

fn try_spawn_anomaly(world: &mut World, pos: GridPosition) {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.05) {
        let anomaly_type = match rng.gen_range(0..3) {
            0 => AnomalyType::Ruins,
            1 => AnomalyType::Geode,
            _ => AnomalyType::StrangeFlora,
        };

        world.spawn((
            Anomaly {
                anomaly_type,
                reward_amount: 50.0,
            },
            ScanProgress::default(),
            pos,
        ));

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("Discovery: Unearthed an Anomaly!");
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_colony_resources_fields() {
        let resources = ColonyResources::default();
        // Starting resources: enough to bootstrap the colony
        assert!((resources.food - 10.0).abs() < f32::EPSILON);
        assert!((resources.wood - 15.0).abs() < f32::EPSILON);
        assert!((resources.stone - 5.0).abs() < f32::EPSILON);
        assert!((resources.tools - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mining_progress_component() {
        let progress = MiningProgress {
            current: 0.0,
            max: 100.0,
        };
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
        assert!((progress.max - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_increments_progress() {
        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Setup Resources
        world.insert_resource(ColonyResources::default());

        // Spawn Designation with MiningProgress
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Perform work (simulate 1 tick of work)
        mine_rock(&mut world, designation, 1.0);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!((progress.current - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_completion() {
        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 9.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Complete the work
        mine_rock(&mut world, designation, 1.0);

        // 1. Entity should be despawned (Designation removed)
        assert!(world.get_entity(designation).is_err());

        // 2. Terrain should be Dirt
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. ResourceItem should be spawned
        let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        assert!(!items.is_empty(), "Should spawn ResourceItem");
        assert_eq!(items[0].resource_type, ResourceType::Stone);

        // 4. Resources should NOT be credited immediately
        let resources = world.resource::<ColonyResources>();
        // Default stone is 5.0. Should still be 5.0.
        assert!(
            (resources.stone - 5.0).abs() < f32::EPSILON,
            "Resources should not increase until hauled"
        );

        // 5. Check log
        let log = world.resource::<MessageLog>();
        assert!(!log.messages.is_empty());
        assert_eq!(log.messages[0].text, "Mined Stone (Needs Hauling)");
    }

    #[test]
    fn test_mine_rock_ignores_non_rock() {
        let mut world = World::new();
        // Setup Grass tile (cannot mine grass for stone)
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        mine_rock(&mut world, designation, 5.0);

        // Should not progress
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mine_rock_no_position() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                // No GridPosition
            ))
            .id();

        mine_rock(&mut world, designation, 1.0);

        // Should just return, no panic
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mining_progress_is_complete() {
        let p = MiningProgress {
            current: 10.0,
            max: 10.0,
        };
        assert!(p.is_complete());

        let p2 = MiningProgress {
            current: 5.0,
            max: 10.0,
        };
        assert!(!p2.is_complete());
    }

    #[test]
    fn test_forestry_progress_component() {
        let progress = ForestryProgress {
            current: 0.0,
            max: 50.0,
        };
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.max, 50.0);
    }

    #[test]
    fn test_chop_tree_increments_progress() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                ForestryProgress {
                    current: 0.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Perform work
        chop_tree(&mut world, designation, 1.0);

        let progress = world.get::<ForestryProgress>(designation).unwrap();
        assert_eq!(progress.current, 1.0);
    }

    #[test]
    fn test_chop_tree_completion() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                ForestryProgress {
                    current: 9.0,
                    max: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Complete work
        chop_tree(&mut world, designation, 1.0);

        // 1. Entity should be despawned
        assert!(world.get_entity(designation).is_err());

        // 2. Terrain should be Dirt (cleared land)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. ResourceItem should be spawned
        let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        assert!(!items.is_empty(), "Should spawn ResourceItem");
        assert_eq!(items[0].resource_type, ResourceType::Wood);

        // 4. Resources should NOT be credited immediately
        let resources = world.resource::<ColonyResources>();
        // Default wood is 15.0. Should still be 15.0.
        assert!(
            (resources.wood - 15.0).abs() < f32::EPSILON,
            "Resources should not increase until hauled"
        );

        // 5. Check log
        let log = world.resource::<MessageLog>();
        assert!(!log.messages.is_empty());
        assert_eq!(log.messages[0].text, "Chopped Tree (Needs Hauling)");
    }

    #[test]
    fn test_add_wood_clamps_to_max() {
        let mut resources = ColonyResources {
            max_wood: 100.0,
            wood: 90.0,
            ..Default::default()
        };
        resources.add_wood(20.0);
        assert!((resources.wood - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_add_wood_clamps_to_zero() {
        let mut resources = ColonyResources {
            wood: 10.0,
            ..Default::default()
        };
        resources.add_wood(-20.0);
        assert!(
            (resources.wood - 0.0).abs() < f32::EPSILON,
            "Resources should not be negative"
        );
    }

    #[test]
    fn test_try_deduct_success() {
        let mut resources = ColonyResources {
            wood: 10.0,
            ..Default::default()
        };
        let cost = ColonyResources {
            wood: 5.0,
            ..Default::default()
        };

        let result = resources.try_deduct(&cost);
        assert!(result);
        assert!((resources.wood - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_try_deduct_failure() {
        let mut resources = ColonyResources {
            wood: 3.0,
            ..Default::default()
        };
        let cost = ColonyResources {
            wood: 5.0,
            ..Default::default()
        };

        let result = resources.try_deduct(&cost);
        assert!(!result);
        // Should be unchanged
        assert!((resources.wood - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_try_deduct_atomic_check() {
        let mut resources = ColonyResources {
            wood: 100.0,
            stone: 10.0,
            ..Default::default()
        };
        // Has enough wood (50 < 100), but not enough stone (20 > 10)
        let cost = ColonyResources {
            wood: 50.0,
            stone: 20.0,
            ..Default::default()
        };

        let success = resources.try_deduct(&cost);
        assert!(!success);

        // Ensure NOTHING was deducted (atomic)
        assert!(
            (resources.wood - 100.0).abs() < f32::EPSILON,
            "Wood should not be deducted"
        );
        assert!(
            (resources.stone - 10.0).abs() < f32::EPSILON,
            "Stone should not be deducted"
        );
    }
}
