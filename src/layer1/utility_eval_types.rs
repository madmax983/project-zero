#![allow(clippy::trivially_copy_pass_by_ref)]
use crate::layer1::chemical::ChemicalState;
use crate::layer1::combat::Drafted;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::factions::{FactionData, FactionId, FactionMember};
use crate::layer1::health::Health;
use crate::layer1::hobby::Hobby;
use crate::layer1::items::{CarryingItem, Equipment, ItemType};
use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::memetic::MemeticCarrier;
use crate::layer1::needs::Needs;
use crate::layer1::penal::PenalLabor;
use crate::layer1::resources::{Carrying, ColonyResources, ResourceType};
use crate::layer1::stress::{BREAKDOWN_TICKS_REQUIRED, Breakdown, StressTracker};
use crate::layer1::taboo::TabooState;
use crate::layer1::traits::Traits;
use crate::layer1::unrest::MentalState;
use crate::layer1::utility_types::{
    ActionType, HobbyType, PopAction, UtilityWeights, calculate_context_score,
};
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use std::collections::HashMap;

/// Evaluates the utility of being idle.
///
/// Idle is a low-priority fallback action. Pops should prefer productive
/// activities (work, eating, resting) over standing around.
#[must_use]
pub const fn evaluate_idle(_needs: &Needs) -> f32 {
    0.05
}

/// Query data for pop evaluation.
///
/// This struct replaces the large tuple query in `evaluate_actions_system`,
/// improving readability and maintainability.
#[derive(QueryData)]
#[query_data(derive(Debug))]
pub struct PopEvaluationQuery {
    pub entity: Entity,
    pub pos: &'static GridPosition,
    pub needs: &'static Needs,
    pub weights: &'static UtilityWeights,
    pub action: &'static PopAction,
    pub equipment: Option<&'static Equipment>,
    pub carrying: Option<&'static Carrying>,
    pub carrying_item: Option<&'static CarryingItem>,
    pub mental_state: Option<&'static MentalState>,
    pub drafted: Option<&'static Drafted>,
    pub inmate: Option<&'static Inmate>,
    pub faction_member: Option<&'static FactionMember>,
    pub penal_labor: Option<&'static PenalLabor>,
    pub breakdown: Option<&'static Breakdown>,
    pub traits: Option<&'static Traits>,
    pub stress: Option<&'static StressTracker>,
    pub hobby: Option<&'static Hobby>,
    pub chemical: Option<&'static ChemicalState>,
    pub memetic_carrier: Option<&'static MemeticCarrier>,
    pub health: Option<&'static Health>,
}

impl PopEvalData {
    /// Converts a query item into `PopEvalData`.
    pub fn from_query_item(item: PopEvaluationQueryItem<'_>) -> Self {
        Self {
            entity: item.entity,
            pos: *item.pos,
            needs: *item.needs,
            weights: *item.weights,
            action: *item.action,
            equipment: item.equipment.copied(),
            carrying: item.carrying.copied(),
            carrying_item: item.carrying_item.map(|c| c.0),
            mental_state: item.mental_state.copied(),
            drafted: item.drafted.copied(),
            faction_member: item.faction_member.cloned(),
            penal_labor: item.penal_labor.copied(),
            breakdown: item.breakdown.copied(),
            traits: item.traits.cloned(),
            stress: item
                .stress
                .map_or(0.0, |s| s.accumulated_stress / BREAKDOWN_TICKS_REQUIRED),
            hobby_type: item.hobby.map(|comp| comp.hobby_type),
            chemical_state: item.chemical.cloned(),
            is_memetic_carrier: item.memetic_carrier.is_some(),
            health: item.health.copied(),
            insulation: 0.0,
            carrying_item_type: None,
        }
    }
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
    pub carrying: Option<Carrying>,
    /// Item currently carried by the pop (as a physical entity), if any.
    pub carrying_item: Option<Entity>,
    /// The ItemType of the carried item, if any.
    pub carrying_item_type: Option<ItemType>,
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
    /// Chemical addiction state, if any.
    pub chemical_state: Option<ChemicalState>,
    /// Whether the pop carries a memetic virus.
    pub is_memetic_carrier: bool,
    /// Health of the pop, if any.
    pub health: Option<Health>,
    /// Current insulation provided by clothing.
    pub insulation: f32,
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
    /// Reference to temperature grid (for clothing checks).
    pub temperature_grid: Option<&'a crate::layer1::temperature::TemperatureGrid>,
}

// --- UNIFIED PROXY ---
// Razor's Cut: Replaced 5 repetitive proxy structs with one generic candidate struct.

/// Represents any entity that a Pop might interact with (Workplace, Item, Location).
#[derive(Clone, Debug)]
pub struct ScorableCandidate {
    /// The entity ID.
    pub entity: Entity,
    /// The location on the grid.
    pub pos: GridPosition,
    /// Total capacity (default 1).
    pub capacity: usize,
    /// Current usage (default 0).
    pub usage: usize,
    /// Generic score bonus (e.g., refining progress).
    pub score_bonus: f32,
    /// Type of resource (if an item).
    pub resource_type: Option<ResourceType>,
    /// Type of generic item (if an item entity).
    pub item_type: Option<ItemType>,
}

impl ScorableCandidate {
    /// Creates a simple position-based candidate.
    pub const fn new(entity: Entity, pos: GridPosition) -> Self {
        Self {
            entity,
            pos,
            capacity: 1,
            usage: 0,
            score_bonus: 0.0,
            resource_type: None,
            item_type: None,
        }
    }

    /// Creates a candidate with capacity.
    pub const fn with_capacity(
        entity: Entity,
        pos: GridPosition,
        capacity: usize,
        usage: usize,
    ) -> Self {
        Self {
            entity,
            pos,
            capacity,
            usage,
            score_bonus: 0.0,
            resource_type: None,
            item_type: None,
        }
    }
}

/// Generic evaluation function for finding the best candidate.
///
/// Replaces repetitive loops in individual `evaluate_*` functions.
///
/// # Arguments
/// * `pop_pos` - The position of the Pop.
/// * `weights` - The Pop's utility weights.
/// * `candidates` - The list of candidates to evaluate.
/// * `base_utility` - The base score for this action.
///
/// # Returns
/// * `Some((utility, entity))` if a valid candidate is found.
#[must_use]
pub fn evaluate_candidates(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    base_utility: f32,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    for candidate in candidates {
        let context = calculate_context_score(
            pop_pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            weights,
        );

        let utility = (base_utility + candidate.score_bonus) * context;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, candidate.entity));
        }
    }
    best
}

/// Reusable buffer for `evaluate_actions_system` to avoid allocations.
#[derive(Resource, Default)]
pub struct UtilityAIBuffer {
    /// Buffer for pop data.
    pub pop_data: Vec<PopEvalData>,

    /// Buffer for evaluation results (Actions decided).
    /// Used to avoid re-allocating the results vector every frame.
    pub results: Vec<Option<(ActionType, f32, Option<Entity>)>>,

    // Candidate Buffers
    /// Buffer for farm candidates.
    pub farms: Vec<ScorableCandidate>,
    /// Buffer for housing candidates.
    pub housing: Vec<ScorableCandidate>,
    /// Buffer for tavern candidates.
    pub taverns: Vec<ScorableCandidate>,
    /// Buffer for library candidates.
    pub libraries: Vec<ScorableCandidate>,
    /// Buffer for refining candidates.
    pub refining: Vec<ScorableCandidate>,
    /// Buffer for work designation candidates.
    pub work_designations: Vec<ScorableCandidate>,
    /// Buffer for repair designation candidates.
    pub repair_designations: Vec<ScorableCandidate>,
    /// Buffer for tame designation candidates.
    pub tame_designations: Vec<ScorableCandidate>,
    /// Buffer for loose item candidates.
    pub items: Vec<ScorableCandidate>,
    /// Buffer for loose generic item candidates.
    pub item_entities: Vec<ScorableCandidate>,
    /// Buffer for stockpile candidates.
    pub stockpiles: Vec<ScorableCandidate>,
    /// Buffer for anomaly candidates.
    pub anomalies: Vec<ScorableCandidate>,
    /// Buffer for hospital candidates.
    pub hospitals: Vec<ScorableCandidate>,
    /// Buffer for corpse candidates.
    pub corpses: Vec<ScorableCandidate>,
    /// Buffer for grave candidates.
    pub graves: Vec<ScorableCandidate>,
    /// Buffer for structure candidates needing repair.
    pub repair_structures: Vec<ScorableCandidate>,
    /// Buffer for wanted criminals.
    pub wanted_criminals: Vec<ScorableCandidate>,
    /// Buffer for office candidates.
    pub offices: Vec<ScorableCandidate>,
    /// Buffer for walls (Memetic Sigil targets).
    pub walls: Vec<ScorableCandidate>,
    /// Buffer for enemies (Fauna/Flora for drafted pops).
    pub enemies: Vec<ScorableCandidate>,
    /// Buffer for all structures (Mental Break targets).
    pub all_structures: Vec<ScorableCandidate>,
    /// Buffer for shower candidates.
    pub showers: Vec<ScorableCandidate>,
    /// Buffer for hum source candidates.
    pub hum_sources: Vec<ScorableCandidate>,
    /// Buffer for Gene Banks.
    pub gene_banks: Vec<ScorableCandidate>,
    /// Buffer for ghost code residue candidates (Purge job).
    pub residues: Vec<ScorableCandidate>,
}
