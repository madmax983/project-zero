use crate::layer1::combat::Drafted;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::factions::{FactionData, FactionId, FactionMember};
use crate::layer1::items::{Equipment, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::penal::PenalLabor;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::stress::Breakdown;
use crate::layer1::taboo::TabooState;
use crate::layer1::traits::Traits;
use crate::layer1::unrest::MentalState;
use crate::layer1::utility_types::{HobbyType, PopAction, UtilityWeights};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Evaluates the utility of being idle.
///
/// Idle is a low-priority fallback action. Pops should prefer productive
/// activities (work, eating, resting) over standing around.
#[must_use]
pub const fn evaluate_idle(_needs: &Needs) -> f32 {
    0.05
}

/// Data bundle for pop evaluation, optimized for copy.
#[derive(Clone, Debug)]
pub struct PopEvalData {
    /// The entity ID of the pop.
    pub entity: Entity,
    /// The current grid position of the pop.
    pub pos: GridPosition,
    /// The current needs (hunger, rest, etc.) of the pop.
    pub needs: Needs,
    /// The personality/memory weights for decision making.
    pub weights: UtilityWeights,
    /// The current action state.
    pub action: PopAction,
    /// Equipment held by the pop, if any.
    pub equipment: Option<Equipment>,
    /// Resource currently carried by the pop, if any.
    pub carrying: Option<crate::layer1::resources::Carrying>,
    /// Item currently carried by the pop (as a physical entity), if any.
    pub carrying_item: Option<Entity>,
    /// Current mental state (e.g., Broken, Dazed), if any.
    pub mental_state: Option<MentalState>,
    /// Draft status (combat mode), if any.
    pub drafted: Option<Drafted>,
    /// Faction membership details, if any.
    pub faction_member: Option<FactionMember>,
    /// Penal labor status, if any.
    pub penal_labor: Option<PenalLabor>,
    /// Breakdown status, if any.
    pub breakdown: Option<Breakdown>,
    /// Personality traits, if any.
    pub traits: Option<Traits>,
    /// Accumulated stress (normalized 0.0-1.0), derived from `StressTracker`.
    pub stress: f32,
    /// Assigned hobby type, if any.
    pub hobby_type: Option<HobbyType>,
}

/// Context data for utility evaluation (resources, time, etc.)
pub struct WorldContext<'a> {
    /// Reference to global colony resources (food, wood, etc.).
    pub resources: &'a ColonyResources,
    /// Reference to the day/night cycle (for shift checks).
    #[allow(dead_code)]
    pub cycle: &'a DayNightCycle,
    /// Reference to current taboo/law state.
    pub taboo: &'a TabooState,
    /// Reference to faction data (for strike checks).
    pub factions: Option<&'a HashMap<FactionId, FactionData>>,
    /// Reference to zone grid (for sanctuary checks).
    pub zone_grid: &'a crate::layer1::zone::ZoneGrid,
}

// --- PROXY TYPES ---
// These lightweight structs allow us to pre-filter and gather candidates
// into flat vectors, avoiding repeated query iterations and complex
// component access during the hot loop of utility evaluation.

/// Generic proxy for entities with position.
#[derive(Clone, Copy, Debug)]
pub struct PositionProxy {
    /// The entity.
    pub entity: Entity,
    /// The location.
    pub pos: GridPosition,
}

/// Generic proxy for entities with capacity (e.g., buildings).
#[derive(Clone, Copy, Debug)]
pub struct CapacityProxy {
    /// The entity.
    pub entity: Entity,
    /// The location.
    pub pos: GridPosition,
    /// Total capacity.
    pub capacity: usize,
    /// Current usage.
    pub usage: usize,
}

/// Proxy struct for Refining Buildings.
#[derive(Clone, Copy, Debug)]
pub struct RefiningProxy {
    /// The building entity.
    pub entity: Entity,
    /// The location of the building.
    pub pos: GridPosition,
    /// Current progress of the refining batch.
    pub progress_current: f32,
}

/// Proxy struct for Resource Items.
#[derive(Clone, Copy, Debug)]
pub struct ItemProxy {
    /// The item entity.
    pub entity: Entity,
    /// The location of the item.
    pub pos: GridPosition,
    /// The type of resource.
    pub resource_type: ResourceType,
}

/// Proxy struct for Generic Items (Entities).
#[derive(Clone, Debug)]
pub struct ItemEntityProxy {
    /// The item entity.
    pub entity: Entity,
    /// The location of the item.
    pub pos: GridPosition,
    /// The type of item.
    pub item_type: ItemType,
}

/// Reusable buffer for `evaluate_actions_system` to avoid allocations.
#[derive(Resource, Default)]
pub struct UtilityAIBuffer {
    /// Buffer for pop data.
    pub pop_data: Vec<PopEvalData>,

    // Candidate Buffers
    /// Buffer for farm candidates.
    pub farms: Vec<CapacityProxy>,
    /// Buffer for housing candidates.
    pub housing: Vec<CapacityProxy>,
    /// Buffer for tavern candidates.
    pub taverns: Vec<CapacityProxy>,
    /// Buffer for library candidates.
    pub libraries: Vec<CapacityProxy>,
    /// Buffer for refining candidates.
    pub refining: Vec<RefiningProxy>,
    /// Buffer for work designation candidates.
    pub work_designations: Vec<PositionProxy>,
    /// Buffer for repair designation candidates.
    pub repair_designations: Vec<PositionProxy>,
    /// Buffer for tame designation candidates.
    pub tame_designations: Vec<PositionProxy>,
    /// Buffer for loose item candidates.
    pub items: Vec<ItemProxy>,
    /// Buffer for loose generic item candidates.
    pub item_entities: Vec<ItemEntityProxy>,
    /// Buffer for stockpile candidates.
    pub stockpiles: Vec<PositionProxy>,
    /// Buffer for anomaly candidates.
    pub anomalies: Vec<PositionProxy>,
    /// Buffer for hospital candidates.
    pub hospitals: Vec<CapacityProxy>,
    /// Buffer for corpse candidates.
    pub corpses: Vec<PositionProxy>,
    /// Buffer for grave candidates.
    pub graves: Vec<PositionProxy>,
    /// Buffer for structure candidates needing repair.
    pub repair_structures: Vec<PositionProxy>,
    /// Buffer for wanted criminals.
    pub wanted_criminals: Vec<PositionProxy>,
    /// Buffer for office candidates.
    pub offices: Vec<CapacityProxy>,
}
