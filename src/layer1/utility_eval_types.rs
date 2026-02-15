use crate::layer1::combat::Drafted;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::factions::{FactionData, FactionId, FactionMember};
use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::penal::PenalLabor;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::taboo::TabooState;
use crate::layer1::unrest::MentalState;
use crate::layer1::utility_types::{ActionType, PopAction, UtilityWeights};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Tracks the outcome of a plan for learning purposes.
///
/// When an action completes (success or failure), this component is used
/// to update the [`UtilityWeights`].
#[derive(Component, Debug)]
pub struct PlanOutcome {
    /// The action type being tracked.
    pub action: ActionType,
    /// Tick when the action started.
    pub started_at: u64,
    /// Needs state before the action (to measure improvement).
    pub needs_before: Needs,
}

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
    /// Current mental state (e.g., Broken, Dazed), if any.
    pub mental_state: Option<MentalState>,
    /// Draft status (combat mode), if any.
    pub drafted: Option<Drafted>,
    /// Faction membership details, if any.
    pub faction_member: Option<FactionMember>,
    /// Penal labor status, if any.
    pub penal_labor: Option<PenalLabor>,
}

/// Context data for utility evaluation (resources, time, etc.)
pub struct WorldContext<'a> {
    /// Reference to global colony resources (food, wood, etc.).
    pub resources: &'a ColonyResources,
    /// Reference to the day/night cycle (for shift checks).
    pub cycle: &'a DayNightCycle,
    /// Reference to current taboo/law state.
    pub taboo: &'a TabooState,
    /// Reference to faction data (for strike checks).
    pub factions: Option<&'a HashMap<FactionId, FactionData>>,
}

// --- PROXY TYPES ---
// These lightweight structs allow us to pre-filter and gather candidates
// into flat vectors, avoiding repeated query iterations and complex
// component access during the hot loop of utility evaluation.

/// Proxy struct for Farms.
#[derive(Clone, Copy, Debug)]
pub struct FarmProxy {
    /// The farm entity.
    pub entity: Entity,
    /// The location of the farm.
    pub pos: GridPosition,
    /// Total worker capacity.
    pub capacity: usize,
    /// Current number of workers.
    pub workers: usize,
}

/// Proxy struct for Housing.
#[derive(Clone, Copy, Debug)]
pub struct HousingProxy {
    /// The housing entity.
    pub entity: Entity,
    /// The location of the house.
    pub pos: GridPosition,
    /// Total resident capacity.
    pub capacity: usize,
    /// Current number of residents.
    pub occupants: usize,
}

/// Proxy struct for Taverns.
#[derive(Clone, Copy, Debug)]
pub struct TavernProxy {
    /// The tavern entity.
    pub entity: Entity,
    /// The location of the tavern.
    pub pos: GridPosition,
    /// Total visitor capacity.
    pub capacity: usize,
    /// Current number of patrons.
    pub patrons: usize,
}

/// Proxy struct for Libraries.
#[derive(Clone, Copy, Debug)]
pub struct LibraryProxy {
    /// The library entity.
    pub entity: Entity,
    /// The location of the library.
    pub pos: GridPosition,
    /// Total researcher capacity.
    pub capacity: usize,
    /// Current number of researchers.
    pub researchers: usize,
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

/// Proxy struct for Work Designations.
#[derive(Clone, Copy, Debug)]
pub struct WorkDesignationProxy {
    /// The designation entity.
    pub entity: Entity,
    /// The location of the designation.
    pub pos: GridPosition,
}

/// Proxy struct for Repair Designations.
#[derive(Clone, Copy, Debug)]
pub struct RepairDesignationProxy {
    /// The designation entity.
    pub entity: Entity,
    /// The location of the designation.
    pub pos: GridPosition,
}

/// Proxy struct for Tame Designations.
#[derive(Clone, Copy, Debug)]
pub struct TameDesignationProxy {
    /// The designation entity.
    pub entity: Entity,
    /// The location of the designation.
    pub pos: GridPosition,
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

/// Proxy struct for Stockpiles.
#[derive(Clone, Copy, Debug)]
pub struct StockpileProxy {
    /// The stockpile entity.
    pub entity: Entity,
    /// The location of the stockpile.
    pub pos: GridPosition,
}

/// Proxy struct for Anomalies.
#[derive(Clone, Copy, Debug)]
pub struct AnomalyProxy {
    /// The anomaly entity.
    pub entity: Entity,
    /// The location of the anomaly.
    pub pos: GridPosition,
}

/// Proxy struct for Hospitals.
#[derive(Clone, Copy, Debug)]
pub struct HospitalProxy {
    /// The hospital entity.
    pub entity: Entity,
    /// The location of the hospital.
    pub pos: GridPosition,
    /// Total patient capacity.
    pub capacity: usize,
    /// Current number of patients.
    pub patients: usize,
}

/// Proxy struct for Corpses.
#[derive(Clone, Copy, Debug)]
pub struct CorpseProxy {
    /// The corpse entity.
    pub entity: Entity,
    /// The location of the corpse.
    pub pos: GridPosition,
}

/// Proxy struct for Graves.
#[derive(Clone, Copy, Debug)]
pub struct GraveProxy {
    /// The grave entity.
    pub entity: Entity,
    /// Whether the grave is occupied.
    pub occupied: bool,
}

/// Proxy struct for Structures (needing repair).
#[derive(Clone, Copy, Debug)]
pub struct StructureProxy {
    /// The structure entity.
    pub entity: Entity,
    /// The location of the structure.
    pub pos: GridPosition,
}

/// Reusable buffer for `evaluate_actions_system` to avoid allocations.
#[derive(Resource, Default)]
pub struct UtilityAIBuffer {
    /// Buffer for pop data.
    pub pop_data: Vec<PopEvalData>,

    // Candidate Buffers
    /// Buffer for farm candidates.
    pub farms: Vec<FarmProxy>,
    /// Buffer for housing candidates.
    pub housing: Vec<HousingProxy>,
    /// Buffer for tavern candidates.
    pub taverns: Vec<TavernProxy>,
    /// Buffer for library candidates.
    pub libraries: Vec<LibraryProxy>,
    /// Buffer for refining candidates.
    pub refining: Vec<RefiningProxy>,
    /// Buffer for work designation candidates.
    pub work_designations: Vec<WorkDesignationProxy>,
    /// Buffer for repair designation candidates.
    pub repair_designations: Vec<RepairDesignationProxy>,
    /// Buffer for tame designation candidates.
    pub tame_designations: Vec<TameDesignationProxy>,
    /// Buffer for loose item candidates.
    pub items: Vec<ItemProxy>,
    /// Buffer for stockpile candidates.
    pub stockpiles: Vec<StockpileProxy>,
    /// Buffer for anomaly candidates.
    pub anomalies: Vec<AnomalyProxy>,
    /// Buffer for hospital candidates.
    pub hospitals: Vec<HospitalProxy>,
    /// Buffer for corpse candidates.
    pub corpses: Vec<CorpseProxy>,
    /// Buffer for grave candidates.
    pub graves: Vec<GraveProxy>,
    /// Buffer for structure candidates needing repair.
    pub repair_structures: Vec<StructureProxy>,
}
