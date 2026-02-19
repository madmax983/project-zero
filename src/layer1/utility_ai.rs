//! # Utility AI: The Brain of the Colony
//!
//! This module implements a **Utility-based AI** system (sometimes called "Need-based AI")
//! that drives the behavior of every Pop in the colony.
//!
//! ## The Decision Cycle
//!
//! Every few ticks (configured in [`UtilityConfig`]), a Pop evaluates its options:
//!
//! 1.  **Identify Candidates**: Scans the world for possible actions (e.g., "There is a farm at (10, 5)").
//! 2.  **Score Candidates**: Calculates a utility score (0.0 - 1.0+) for each option based on:
//!     *   **Needs**: "I am hungry" (increases food utility).
//!     *   **Distance**: "It's too far away" (decreases utility via [`calculate_context_score`]).
//!     *   **Personality**: "I hate hauling" (modifiers from traits/memories).
//!     *   **Learning**: "I failed at this last time" (reinforcement learning via [`UtilityWeights`]).
//! 3.  **Select Best**: The action with the highest score wins.
//! 4.  **Commit**: The Pop commits to the action for a duration or until a better option appears.
//!
//! ## Key Components
//!
//! *   [`evaluate_actions_system`]: The main loop that runs the decision cycle.
//! *   [`ActionType`]: The enum of all possible behaviors.
//! *   [`UtilityWeights`]: The "memory" of the Pop, adjusting scores based on past success/failure.
//!
//! ## Performance Architecture
//!
//! The Utility AI system is one of the most computationally expensive parts of the simulation,
//! potentially running for hundreds of pops every tick. To maintain high FPS:
//!
//! 1.  **Staggered Evaluation**: Not every pop thinks every tick. [`UtilityConfig::evaluation_interval`]
//!     spreads the load (e.g., only 1/60th of pops think per frame).
//! 2.  **Allocation-Free Loop**: [`evaluate_actions_system`] reuses a single `UtilityAIBuffer`
//!     resource. It clears the buffer instead of dropping it, preventing thousands of
//!     `Vec::new()` calls per frame.
//! 3.  **Entity Iteration**: We use `Query::iter` (which is fast) rather than random access.
//! 4.  **Proxy Buffering**: We pre-collect candidate entities (Farms, Stockpiles, etc.) into flat
//!     vectors at the start of the system. This avoids repeatedly querying the world or creating
//!     iterators for every single Pop, converting an O(N*M) query operation into O(M) query + O(N*M)
//!     vector iteration (which is much faster due to cache locality and no ECS overhead).

use crate::layer1::actions::explore::evaluate_explore;
use crate::layer1::actions::farm::evaluate_farm;
use crate::layer1::actions::fetch_clothing::evaluate_fetch_clothing;
use crate::layer1::actions::fetch_tool::evaluate_fetch_tool;
use crate::layer1::actions::fight::evaluate_drafted_behavior;
use crate::layer1::actions::funeral::evaluate_bury_corpse;
use crate::layer1::actions::haul::evaluate_haul;
use crate::layer1::actions::hunger::evaluate_satisfy_hunger;
use crate::layer1::actions::medical::evaluate_seek_medical_care;
use crate::layer1::actions::mental_break::evaluate_mental_break;
use crate::layer1::actions::refine::evaluate_refine;
use crate::layer1::actions::repair::evaluate_repair;
use crate::layer1::actions::research::evaluate_research;
use crate::layer1::actions::rest::evaluate_satisfy_rest;
use crate::layer1::actions::social::evaluate_socialize;
use crate::layer1::actions::work::evaluate_work;
use crate::layer1::building::{Building, ShiftSchedule};
use crate::layer1::combat::Drafted;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::farm::Farm;
use crate::layer1::funeral::{Corpse, Grave};
use crate::layer1::hobby::{Hobby, evaluate_hobby};
use crate::layer1::housing::Housing;
use crate::layer1::husbandry::evaluate_tame;
use crate::layer1::items::Equipment;
use crate::layer1::justice::{Inmate, Wanted, evaluate_warden_action};
use crate::layer1::map::GridPosition;
use crate::layer1::medical::Hospital;
use crate::layer1::needs::Needs;
use crate::layer1::penal::PenalLabor;
use crate::layer1::refining::get_refining_recipe;
use crate::layer1::resources::{ColonyResources, RefiningProgress, ResourceItem};
use crate::layer1::science::Anomaly;
use crate::layer1::social::Tavern;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::stress::{Breakdown, StressTracker, BREAKDOWN_TICKS_REQUIRED};
use crate::layer1::structure::{DeferMaintenance, Structure};
use crate::layer1::tech::Library;
use crate::layer1::traits::Trait;
use crate::layer1::unrest::MentalState;
use crate::layer1::utility_eval_types::{
    CapacityProxy, ItemProxy, PlanOutcome, PopEvalData, PositionProxy, RefiningProxy,
    UtilityAIBuffer, WorldContext, evaluate_idle,
};
pub use crate::layer1::utility_types::{
    ActionType, PopAction, StartPlan, UtilityConfig, UtilityWeights, manhattan_distance,
};
use crate::layer1::zone::ZoneGrid;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// System to update commitment timers.
/// Increments the committed-tick counter for every pop's action.
///
/// Uses `par_iter_mut` for parallel processing across entities.
pub fn update_action_timer_system(mut query: Query<&mut PopAction>) {
    query.par_iter_mut().for_each(|mut action| {
        action.ticks_committed = action.ticks_committed.saturating_add(1);
    });
}

/// Helper struct to track the best action found so far.
struct CandidateEvaluator {
    action: ActionType,
    utility: f32,
    target: Option<Entity>,
}

impl CandidateEvaluator {
    /// Creates a new evaluator with an initial utility score.
    const fn new(initial_utility: f32) -> Self {
        Self {
            action: ActionType::Idle,
            utility: initial_utility,
            target: None,
        }
    }

    /// Updates the best action if the candidate has higher utility.
    fn consider(&mut self, action: ActionType, utility: f32, target: Option<Entity>) {
        if utility > self.utility {
            self.action = action;
            self.utility = utility;
            self.target = target;
        }
    }

    /// Returns the best action found.
    const fn result(self) -> (ActionType, f32, Option<Entity>) {
        (self.action, self.utility, self.target)
    }
}

fn is_pop_striking(data: &PopEvalData, context: &WorldContext) -> bool {
    context.factions.as_ref().is_some_and(|map| {
        data.faction_member.as_ref().is_some_and(|member| {
            member.faction_id.is_some_and(|fid| {
                map.get(&fid).is_some_and(|data| {
                    data.state == crate::layer1::factions::FactionState::Striking
                })
            })
        })
    })
}

#[allow(clippy::collapsible_if)]
fn evaluate_group_survival(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    world: &World,
) {
    let pop_entity = data.entity;
    let pop_pos = data.pos;
    let needs = data.needs;
    let weights = data.weights;

    // Evaluate Hunger
    if let Some((utility, target)) =
        evaluate_satisfy_hunger(pop_pos, &needs, &weights, &buffer.farms)
    {
        evaluator.consider(ActionType::SatisfyHunger, utility, Some(target));
    }

    // Evaluate SatisfyRest
    if let Some((utility, target)) =
        evaluate_satisfy_rest(pop_pos, &needs, &weights, &buffer.housing)
    {
        evaluator.consider(ActionType::SatisfyRest, utility, Some(target));
    }

    // Check Health
    let health = world.get::<crate::layer1::health::Health>(pop_entity);

    // Evaluate SeekMedicalCare
    if let Some(health) = health {
        if let Some((utility, target)) =
            evaluate_seek_medical_care(pop_pos, &needs, *health, &weights, &buffer.hospitals)
        {
            evaluator.consider(ActionType::SeekMedicalCare, utility, Some(target));
        }
    }
}

fn evaluate_group_social(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) {
    let pop_pos = data.pos;
    let needs = data.needs;
    let weights = data.weights;
    let is_penal = data.penal_labor.is_some();

    if is_penal {
        return;
    }

    // Evaluate Socialize
    if let Some((utility, target)) = evaluate_socialize(pop_pos, &needs, &weights, &buffer.taverns)
    {
        evaluator.consider(ActionType::Socialize, utility, Some(target));
    }
}

#[allow(clippy::collapsible_if)]
fn evaluate_group_work(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    context: &WorldContext,
    is_striking: bool,
) {
    let pop_pos = data.pos;
    let weights = data.weights;
    let is_penal = data.penal_labor.is_some();
    let is_feral = data
        .traits
        .as_ref()
        .is_some_and(|t| t.0.contains(&Trait::Feral));

    if is_striking {
        return;
    }

    // Evaluate Work
    if let Some((utility, target)) = evaluate_work(pop_pos, &weights, &buffer.work_designations) {
        let penalty = crate::layer1::taboo::evaluate_taboo_penalty(ActionType::Work, context.taboo);
        let bonus = if is_penal {
            1.0 // High priority for penal labor
        } else {
            0.0
        };
        evaluator.consider(ActionType::Work, utility + penalty + bonus, Some(target));
    }

    // Evaluate Refine
    if !is_penal {
        if let Some((utility, target)) = evaluate_refine(pop_pos, &weights, &buffer.refining) {
            evaluator.consider(ActionType::Refine, utility, Some(target));
        }
    }

    // Evaluate Farm
    if !is_penal {
        if let Some((utility, target)) = evaluate_farm(pop_pos, &weights, &buffer.farms) {
            evaluator.consider(ActionType::Farm, utility, Some(target));
        }
    }

    // Evaluate Research
    if !is_penal && !is_feral {
        if let Some((utility, target)) =
            evaluate_research(pop_pos, &weights, context.resources, &buffer.libraries)
        {
            evaluator.consider(ActionType::Research, utility, Some(target));
        }
    }

    // Evaluate Tame
    if let Some((utility, target)) = evaluate_tame(&pop_pos, &weights, &buffer.tame_designations) {
        evaluator.consider(ActionType::Tame, utility, Some(target));
    }

    // Evaluate Warden
    if !is_penal {
        if let Some((utility, target)) =
            evaluate_warden_action(&pop_pos, &buffer.wanted_criminals, context.zone_grid)
        {
            evaluator.consider(ActionType::Warden, utility, Some(target));
        }
    }
}

fn evaluate_group_logistics(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    context: &WorldContext,
    is_striking: bool,
) {
    let pop_pos = data.pos;
    let weights = data.weights;
    let equipment_opt = data.equipment;

    if is_striking {
        return;
    }

    // Evaluate FetchTool
    let equipment = equipment_opt.unwrap_or_default();
    if let Some((utility, target)) =
        evaluate_fetch_tool(pop_pos, &equipment, context.resources, &buffer.stockpiles)
    {
        evaluator.consider(ActionType::FetchTool, utility, Some(target));
    }

    // Evaluate FetchClothing
    if let Some((utility, target)) =
        evaluate_fetch_clothing(pop_pos, &equipment, context.resources, &buffer.stockpiles)
    {
        evaluator.consider(ActionType::FetchClothing, utility, Some(target));
    }

    // Evaluate Repair
    if let Some((utility, target)) = evaluate_repair(
        pop_pos,
        &weights,
        &buffer.repair_designations,
        &buffer.repair_structures,
    ) {
        evaluator.consider(ActionType::Repair, utility, Some(target));
    }

    // Evaluate Haul
    if let Some((utility, target)) = evaluate_haul(
        pop_pos,
        &weights,
        &buffer.items,
        &buffer.stockpiles,
        context.resources,
        data.carrying,
    ) {
        evaluator.consider(ActionType::Haul, utility, Some(target));
    }

    // Evaluate BuryCorpse
    if let Some((utility, target)) =
        evaluate_bury_corpse(pop_pos, &buffer.corpses, &buffer.graves, &weights)
    {
        evaluator.consider(ActionType::BuryCorpse, utility, Some(target));
    }
}

fn evaluate_group_exploration(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    is_striking: bool,
) {
    let pop_pos = data.pos;
    let weights = data.weights;
    let is_penal = data.penal_labor.is_some();

    if is_striking || is_penal {
        return;
    }

    // Evaluate Explore
    if let Some((utility, target)) = evaluate_explore(pop_pos, &weights, &buffer.anomalies) {
        evaluator.consider(ActionType::Explore, utility, Some(target));
    }
}

fn evaluate_group_leisure(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
) {
    if let Some(hobby_type) = data.hobby_type {
        let utility = evaluate_hobby(data, hobby_type);
        evaluator.consider(ActionType::Hobby, utility, None);
    }
}

/// Helper function to evaluate all potential actions for a single Pop.
///
/// Returns the best `(ActionType, Utility, Target)`.
#[allow(clippy::too_many_lines, clippy::collapsible_if)]
pub(crate) fn evaluate_single_pop(
    buffer: &UtilityAIBuffer,
    world: &mut World,
    data: &PopEvalData,
    context: &WorldContext,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check for Mental Break (Returns early)
    if let Some((action, utility, target)) = evaluate_mental_break(data, world) {
        return (action, utility, target);
    }

    // 2. Check for Drafted (Returns early)
    if let Some((action, utility, target)) = evaluate_drafted_behavior(data, world) {
        return (action, utility, target);
    }

    // 3. Normal evaluation (undrafted, sane)
    let mut evaluator = CandidateEvaluator::new(evaluate_idle(&data.needs));
    let is_striking = is_pop_striking(data, context);

    evaluate_group_survival(&mut evaluator, data, buffer, world);
    evaluate_group_social(&mut evaluator, data, buffer);
    evaluate_group_leisure(&mut evaluator, data);
    evaluate_group_work(&mut evaluator, data, buffer, context, is_striking);
    evaluate_group_logistics(&mut evaluator, data, buffer, context, is_striking);
    evaluate_group_exploration(&mut evaluator, data, buffer, is_striking);

    evaluator.result()
}

fn populate_buffer_buildings(
    world: &mut World,
    buffer: &mut UtilityAIBuffer,
    context: &WorldContext,
) {
    let cycle = context.cycle;
    let resources = context.resources;

    // Farms
    buffer.farms.clear();
    let mut farm_query = world.query::<(Entity, &GridPosition, &Farm, Option<&ShiftSchedule>)>();
    for (entity, pos, farm, schedule) in farm_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        if farm.workers.len() >= farm.capacity {
            continue;
        }
        buffer.farms.push(CapacityProxy {
            entity,
            pos: *pos,
            capacity: farm.capacity,
            usage: farm.workers.len(),
        });
    }

    // Housing
    buffer.housing.clear();
    let mut housing_query = world.query::<(Entity, &GridPosition, &Housing)>();
    for (entity, pos, housing) in housing_query.iter(world) {
        if housing.residents.len() >= housing.capacity {
            continue;
        }
        buffer.housing.push(CapacityProxy {
            entity,
            pos: *pos,
            capacity: housing.capacity,
            usage: housing.residents.len(),
        });
    }

    // Taverns
    buffer.taverns.clear();
    let mut tavern_query = world.query::<(Entity, &GridPosition, &Tavern)>();
    for (entity, pos, tavern) in tavern_query.iter(world) {
        if tavern.visitors.len() >= tavern.capacity {
            continue;
        }
        buffer.taverns.push(CapacityProxy {
            entity,
            pos: *pos,
            capacity: tavern.capacity,
            usage: tavern.visitors.len(),
        });
    }

    // Libraries
    buffer.libraries.clear();
    let mut library_query =
        world.query::<(Entity, &GridPosition, &Library, Option<&ShiftSchedule>)>();
    for (entity, pos, _library, schedule) in library_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        buffer.libraries.push(CapacityProxy {
            entity,
            pos: *pos,
            capacity: 5,
            usage: 0,
        });
    }

    // Refining
    buffer.refining.clear();
    let mut refine_query = world.query::<(
        Entity,
        &GridPosition,
        &Building,
        &RefiningProgress,
        Option<&ShiftSchedule>,
    )>();
    for (entity, pos, building, progress, schedule) in refine_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }

        // Check recipe affordability (Global check)
        let (can_afford, _, _) = get_refining_recipe(building.building_type, resources);
        if !can_afford {
            continue;
        }

        buffer.refining.push(RefiningProxy {
            entity,
            pos: *pos,
            progress_current: progress.current,
        });
    }

    // Hospitals
    buffer.hospitals.clear();
    let mut hospital_query = world.query::<(Entity, &GridPosition, &Hospital)>();
    for (entity, pos, _) in hospital_query.iter(world) {
        buffer.hospitals.push(CapacityProxy {
            entity,
            pos: *pos,
            capacity: 10,
            usage: 0,
        });
    }
}

fn populate_buffer_designations(world: &mut World, buffer: &mut UtilityAIBuffer) {
    // Designations (Work, Repair, Tame)
    buffer.work_designations.clear();
    buffer.repair_designations.clear();
    buffer.tame_designations.clear();
    let mut des_query = world.query::<(Entity, &GridPosition, &Designation)>();
    for (entity, pos, des) in des_query.iter(world) {
        match des.designation_type {
            DesignationType::Repair => {
                buffer
                    .repair_designations
                    .push(PositionProxy { entity, pos: *pos });
            }
            DesignationType::Tame => {
                buffer
                    .tame_designations
                    .push(PositionProxy { entity, pos: *pos });
            }
            _ => {
                buffer
                    .work_designations
                    .push(PositionProxy { entity, pos: *pos });
            }
        }
    }
}

fn populate_buffer_items_and_misc(world: &mut World, buffer: &mut UtilityAIBuffer) {
    // Items
    buffer.items.clear();
    let mut item_query = world.query::<(Entity, &GridPosition, &ResourceItem)>();
    for (entity, pos, item) in item_query.iter(world) {
        buffer.items.push(ItemProxy {
            entity,
            pos: *pos,
            resource_type: item.resource_type,
        });
    }

    // Stockpiles
    buffer.stockpiles.clear();
    let mut stock_query = world.query::<(Entity, &GridPosition, &Stockpile)>();
    for (entity, pos, _) in stock_query.iter(world) {
        buffer.stockpiles.push(PositionProxy { entity, pos: *pos });
    }

    // Anomalies
    buffer.anomalies.clear();
    let mut anomaly_query = world.query::<(Entity, &GridPosition, &Anomaly)>();
    for (entity, pos, _) in anomaly_query.iter(world) {
        buffer.anomalies.push(PositionProxy { entity, pos: *pos });
    }

    // Corpses
    buffer.corpses.clear();
    let mut corpse_query = world.query::<(Entity, &GridPosition, &Corpse)>();
    for (entity, pos, _) in corpse_query.iter(world) {
        buffer.corpses.push(PositionProxy { entity, pos: *pos });
    }

    // Graves
    buffer.graves.clear();
    let mut grave_query = world.query::<(Entity, &GridPosition, &Grave)>();
    for (entity, pos, grave) in grave_query.iter(world) {
        if !grave.occupied {
            buffer.graves.push(PositionProxy { entity, pos: *pos });
        }
    }

    // Structures (Auto-Repair)
    buffer.repair_structures.clear();
    let mut struct_query =
        world.query::<(Entity, &GridPosition, &Structure, Option<&DeferMaintenance>)>();
    for (entity, pos, structure, defer) in struct_query.iter(world) {
        if defer.is_some() {
            continue;
        }
        if (structure.current_hp - structure.max_hp).abs() < f32::EPSILON {
            continue;
        }

        buffer
            .repair_structures
            .push(PositionProxy { entity, pos: *pos });
    }

    // Wanted criminals
    buffer.wanted_criminals.clear();
    let mut wanted_query = world.query::<(Entity, &GridPosition, &Wanted)>();
    for (entity, pos, _) in wanted_query.iter(world) {
        buffer
            .wanted_criminals
            .push(PositionProxy { entity, pos: *pos });
    }
}

/// Populates the AI buffer with candidate entities from the world.
///
/// This function queries the world for all relevant entities (buildings, items, designations)
/// and stores their data in the reusable `UtilityAIBuffer`. This avoids repeated queries
/// during the per-pop evaluation phase.
fn populate_ai_buffer(world: &mut World, buffer: &mut UtilityAIBuffer, context: &WorldContext) {
    populate_buffer_buildings(world, buffer, context);
    populate_buffer_designations(world, buffer);
    populate_buffer_items_and_misc(world, buffer);
}

/// The Main Brain Loop: Decides what every Pop should do next.
///
/// This system runs periodically (every tick, but individual pops only evaluate
/// based on their `evaluation_interval`).
///
/// # The Algorithm
///
/// 1.  **Filter**: Selects Pops who have finished their commitment timer (`ticks_committed`).
/// 2.  **Gather Context**: Pre-fetches all relevant entities (Farms, Stockpiles, etc.)
///     into efficient proxy vectors.
/// 3.  **Evaluate Candidates**:
///     For each Pop, it calls every `evaluate_*` function using the proxies.
/// 4.  **Winner Takes All**: Tracks the single best `(Utility, Action, Target)` tuple.
/// 5.  **Switch**: If the best new utility > current utility + threshold, the Pop switches tasks.
///
/// # Performance Note
/// This system avoids per-Pop query iteration by collecting candidates once per frame.
#[allow(clippy::too_many_lines, clippy::collapsible_if)]
pub fn evaluate_actions_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();

    // Use reusable buffer to avoid repeated heap allocations
    let mut buffer = world
        .remove_resource::<UtilityAIBuffer>()
        .unwrap_or_default();

    // 1. Collect Pop Data
    buffer.pop_data.clear();
    buffer.pop_data.extend(
        world
            .query::<(
                Entity,
                &GridPosition,
                &Needs,
                &UtilityWeights,
                &PopAction,
                Option<&Equipment>,
                Option<&crate::layer1::resources::Carrying>,
                Option<&MentalState>,
                Option<&Drafted>,
                Option<&Inmate>,
                Option<&crate::layer1::factions::FactionMember>,
                Option<&PenalLabor>,
                Option<&Breakdown>,
                (
                    Option<&crate::layer1::traits::Traits>,
                    Option<&StressTracker>,
                    Option<&Hobby>,
                ),
            )>()
            .iter(world)
            .filter(
                |(_, _, _, _, action, _, _, _, _, inmate, _, penal_labor, _, _): &(
                    Entity,
                    &GridPosition,
                    &Needs,
                    &UtilityWeights,
                    &PopAction,
                    Option<&Equipment>,
                    Option<&crate::layer1::resources::Carrying>,
                    Option<&MentalState>,
                    Option<&Drafted>,
                    Option<&Inmate>,
                    Option<&crate::layer1::factions::FactionMember>,
                    Option<&PenalLabor>,
                    Option<&Breakdown>,
                    (
                        Option<&crate::layer1::traits::Traits>,
                        Option<&StressTracker>,
                        Option<&Hobby>,
                    ),
                )| {
                    action.ticks_committed >= config.evaluation_interval
                        && (inmate.is_none() || penal_labor.is_some())
                },
            )
            .map(
                |(e, p, n, w, a, eq, c, m, d, _, fm, pl, b, (t, st, h)): (
                    Entity,
                    &GridPosition,
                    &Needs,
                    &UtilityWeights,
                    &PopAction,
                    Option<&Equipment>,
                    Option<&crate::layer1::resources::Carrying>,
                    Option<&MentalState>,
                    Option<&Drafted>,
                    Option<&Inmate>,
                    Option<&crate::layer1::factions::FactionMember>,
                    Option<&PenalLabor>,
                    Option<&Breakdown>,
                    (
                        Option<&crate::layer1::traits::Traits>,
                        Option<&StressTracker>,
                        Option<&Hobby>,
                    ),
                )| PopEvalData {
                    entity: e,
                    pos: *p,
                    needs: *n,
                    weights: *w,
                    action: *a,
                    equipment: eq.copied(),
                    carrying: c.copied(),
                    mental_state: m.copied(),
                    drafted: d.copied(),
                    faction_member: fm.cloned(),
                    penal_labor: pl.copied(),
                    breakdown: b.copied(),
                    traits: t.cloned(),
                    stress: st.map_or(0.0, |s| s.accumulated_stress / BREAKDOWN_TICKS_REQUIRED),
                    hobby_type: h.map(|comp| comp.hobby_type),
                },
            ),
    );

    // Early exit if no pops need evaluation
    if buffer.pop_data.is_empty() {
        world.insert_resource(buffer);
        return;
    }

    // 2. Initialize Context
    // Optimization: Temporarily remove large resources to avoid cloning them
    let zone_grid_opt = world.remove_resource::<ZoneGrid>();
    let zone_grid_fallback = ZoneGrid::new(1, 1);
    let zone_grid_ref = zone_grid_opt.as_ref().unwrap_or(&zone_grid_fallback);

    let factions_res = world.remove_resource::<crate::layer1::factions::Factions>();
    let factions_data = factions_res.as_ref().map(|f| &f.map);

    let resources = world.resource::<ColonyResources>().clone();
    let cycle = world
        .resource::<crate::layer1::day_night::DayNightCycle>()
        .clone();
    let taboo = world.resource::<crate::layer1::taboo::TabooState>().clone();

    let context = WorldContext {
        resources: &resources,
        cycle: &cycle,
        taboo: &taboo,
        factions: factions_data,
        zone_grid: zone_grid_ref,
    };

    // 3. Populate Proxies (The Optimization)
    // We clear buffers and populate them once, filtering invalid targets early.
    populate_ai_buffer(world, &mut buffer, &context);

    // 4. Evaluate each pop
    for data in &buffer.pop_data {
        let (best_action, best_utility, best_target) =
            evaluate_single_pop(&buffer, world, data, &context);

        // Switch if best exceeds threshold
        if best_utility > data.action.current_utility + config.switch_threshold {
            // Update action
            let mut action = data.action;
            action.current = best_action;
            action.current_utility = best_utility;
            action.ticks_committed = 0;

            // Write back to world
            if let Some(mut pop_action) = world.get_mut::<PopAction>(data.entity) {
                *pop_action = action;
            }

            // Insert StartPlan marker (for HTN system)
            world.entity_mut(data.entity).insert(StartPlan {
                action: best_action,
                target: best_target,
            });
        }
    }

    // Return the buffer to the world
    world.insert_resource(buffer);

    // Restore removed resources
    if let Some(zg) = zone_grid_opt {
        world.insert_resource(zg);
    }
    if let Some(f) = factions_res {
        world.insert_resource(f);
    }
}

/// Updates utility weights based on action outcome.
pub(crate) fn update_weights_from_outcome(
    weights: &mut UtilityWeights,
    action: ActionType,
    success: bool,
    duration: u32,
    config: &UtilityConfig,
) {
    // Track attempt
    let idx = action.as_index();
    weights.action_attempt_count[idx] = weights.action_attempt_count[idx].saturating_add(1);

    if success {
        weights.action_success_count[idx] = weights.action_success_count[idx].saturating_add(1);

        // Successful action: reinforce weights
        if duration < 10 {
            weights.distance_weight += config.learning_rate * 0.1;
        }
        weights.availability_weight += config.learning_rate * 0.05;
    } else {
        // Failed action: reduce weights
        weights.distance_weight -= config.learning_rate * 0.05;
        weights.availability_weight -= config.learning_rate * 0.05;
    }

    // Clamp weights
    weights.distance_weight = weights
        .distance_weight
        .clamp(config.weight_clamp.0, config.weight_clamp.1);
    weights.availability_weight = weights
        .availability_weight
        .clamp(config.weight_clamp.0, config.weight_clamp.1);
    weights.social_weight = weights
        .social_weight
        .clamp(config.weight_clamp.0, config.weight_clamp.1);
}

/// System to track completed plans and trigger learning.
pub fn track_plan_outcomes_system(
    mut completed: Query<(Entity, &PlanOutcome, &Needs, &mut UtilityWeights)>,
    config: Res<UtilityConfig>,
    time: Res<SimulationTime>,
    mut commands: Commands,
) {
    let sim_time = time.tick;

    for (pop_entity, outcome, needs_after, mut weights) in &mut completed {
        let duration = sim_time - outcome.started_at;

        let success = match outcome.action {
            ActionType::SatisfyHunger => (needs_after.hunger - outcome.needs_before.hunger) > 0.05,
            ActionType::SatisfyRest => (needs_after.rest - outcome.needs_before.rest) > 0.05,
            ActionType::Socialize => (needs_after.leisure - outcome.needs_before.leisure) > 0.05,
            ActionType::Work
            | ActionType::Repair
            | ActionType::Explore
            | ActionType::Research
            | ActionType::Haul
            | ActionType::SeekMedicalCare
            | ActionType::BuryCorpse
            | ActionType::FetchTool
            | ActionType::Idle
            | ActionType::Vandalize
            | ActionType::Binge
            | ActionType::Daze
            | ActionType::Fight
            | ActionType::Refine
            | ActionType::Farm
            | ActionType::Warden
            | ActionType::Sleepwalking
            | ActionType::Tame
            | ActionType::FireStarting
            | ActionType::HideInRoom
            | ActionType::SadWander
            | ActionType::FetchClothing
            | ActionType::Surgery
            | ActionType::Charge
            | ActionType::Hobby => true,
        };

        #[allow(clippy::cast_possible_truncation)]
        update_weights_from_outcome(
            &mut weights,
            outcome.action,
            success,
            duration as u32,
            &config,
        );

        commands.entity(pop_entity).remove::<PlanOutcome>();
    }
}

// Re-add tests at the bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    fn setup() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world
    }

    #[test]
    fn test_update_action_timer() {
        let mut world = setup();
        let pop = world.spawn(PopAction::default()).id();

        // Run system
        world.run_system_once(update_action_timer_system).unwrap();

        assert_eq!(world.get::<PopAction>(pop).unwrap().ticks_committed, 1);
    }

    #[test]
    fn test_evaluate_actions_switches_when_threshold_exceeded() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        // Starving pop currently idle
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.8,
                    leisure: 0.8,
                }, // Very hungry!
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.2,
                    ticks_committed: 10, // Past evaluation interval
                },
            ))
            .id();

        // Available farm
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        evaluate_actions_system(&mut world);

        // Should have switched to SatisfyHunger
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::SatisfyHunger);
        assert!(
            action.current_utility > 0.5,
            "Hungry pop should have high hunger utility"
        );
        assert_eq!(action.ticks_committed, 0, "Should reset commitment counter");
    }

    #[test]
    fn test_evaluate_actions_respects_threshold() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig {
            switch_threshold: 0.5, // High threshold
            ..Default::default()
        });
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.6,
                    rest: 0.6,
                    leisure: 0.6,
                }, // Moderate needs
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.4,
                    ticks_committed: 10,
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        evaluate_actions_system(&mut world);

        // Should NOT switch (utility difference < threshold)
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Should not switch with high threshold"
        );
    }

    #[test]
    fn test_update_weights_from_success() {
        let mut weights = UtilityWeights::default();
        let config = UtilityConfig::default();

        let initial_distance = weights.distance_weight;
        let initial_availability = weights.availability_weight;

        update_weights_from_outcome(
            &mut weights,
            ActionType::SatisfyHunger,
            true, // success
            5,    // quick duration
            &config,
        );

        // Weights should increase on success
        assert!(weights.distance_weight > initial_distance);
        assert!(weights.availability_weight > initial_availability);

        // Success should be tracked
        assert_eq!(
            weights.action_attempt_count[ActionType::SatisfyHunger.as_index()],
            1
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            1
        );
    }

    #[test]
    fn test_update_weights_from_failure() {
        let mut weights = UtilityWeights::default();
        let config = UtilityConfig::default();

        let initial_distance = weights.distance_weight;
        let initial_availability = weights.availability_weight;

        update_weights_from_outcome(
            &mut weights,
            ActionType::SatisfyHunger,
            false, // failure
            20,    // slow duration
            &config,
        );

        // Weights should decrease on failure
        assert!(weights.distance_weight < initial_distance);
        assert!(weights.availability_weight < initial_availability);

        // Failure should be tracked
        assert_eq!(
            weights.action_attempt_count[ActionType::SatisfyHunger.as_index()],
            1
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            0
        );
    }

    #[test]
    fn test_weights_clamped_to_range() {
        let mut weights = UtilityWeights::default();
        let config = UtilityConfig::default();

        // Drive weights to minimum
        for _ in 0..100 {
            update_weights_from_outcome(
                &mut weights,
                ActionType::SatisfyHunger,
                false,
                30,
                &config,
            );
        }

        assert!(weights.distance_weight >= config.weight_clamp.0);
        assert!(weights.availability_weight >= config.weight_clamp.0);

        // Drive weights to maximum
        for _ in 0..100 {
            update_weights_from_outcome(&mut weights, ActionType::SatisfyRest, true, 1, &config);
        }

        assert!(weights.distance_weight <= config.weight_clamp.1);
        assert!(weights.availability_weight <= config.weight_clamp.1);
    }

    #[test]
    fn test_track_plan_outcomes_on_success() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.6,
                    rest: 0.8,
                    leisure: 0.8,
                }, // Improved from 0.3
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::SatisfyHunger,
                    started_at: 90,
                    needs_before: Needs {
                        hunger: 0.3,
                        rest: 0.8,
                        leisure: 0.8,
                    },
                },
            ))
            .id();
        // Note: No Plan component = plan completed

        world.run_system_once(track_plan_outcomes_system).unwrap();

        // Should have updated weights
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            1
        );

        // Should have removed PlanOutcome
        assert!(world.get::<PlanOutcome>(pop).is_none());
    }

    #[test]
    fn test_track_plan_outcomes_on_failure() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                }, // No improvement
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::SatisfyHunger,
                    started_at: 90,
                    needs_before: Needs {
                        hunger: 0.3,
                        rest: 0.8,
                        leisure: 0.8,
                    },
                },
            ))
            .id();

        world.run_system_once(track_plan_outcomes_system).unwrap();

        // Should have tracked failure
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(
            weights.action_attempt_count[ActionType::SatisfyHunger.as_index()],
            1
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            0
        );
    }

    #[test]
    fn test_integration_pop_learns_from_experience() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                },
                UtilityWeights::default(),
                PopAction::default(),
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 0 },
            Farm::default(),
        ));

        // Run multiple cycles
        for _ in 0..5 {
            // Simulate successful hunger satisfaction
            world.get_mut::<Needs>(pop).unwrap().hunger = 0.3;
            world.get_mut::<PopAction>(pop).unwrap().ticks_committed = 10;

            evaluate_actions_system(&mut world);

            // Simulate success
            let tick = world.resource::<SimulationTime>().tick;
            world.entity_mut(pop).insert(PlanOutcome {
                action: ActionType::SatisfyHunger,
                started_at: tick,
                needs_before: Needs {
                    hunger: 0.3,
                    rest: 0.8,
                    leisure: 0.8,
                },
            });

            world.get_mut::<Needs>(pop).unwrap().hunger = 0.7; // Success!
            world.resource_mut::<SimulationTime>().tick += 10;

            world.run_system_once(track_plan_outcomes_system).unwrap();
        }

        // Pop should have learned (weights increased)
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert!(
            weights.distance_weight > 1.0 || weights.availability_weight > 1.0,
            "Pop should have learned from successful experiences"
        );
        assert_eq!(
            weights.action_success_count[ActionType::SatisfyHunger.as_index()],
            5
        );
    }

    #[test]
    fn test_track_plan_outcomes_work_action() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs::default(),
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::Work,
                    started_at: 50,
                    needs_before: Needs::default(),
                },
            ))
            .id();

        // Run system
        world.run_system_once(track_plan_outcomes_system).unwrap();

        let weights = world.get::<UtilityWeights>(pop).unwrap();
        // This should be 1 if Work is considered a success when completed
        assert_eq!(
            weights.action_success_count[ActionType::Work.as_index()],
            1,
            "Work action should be counted as success if completed"
        );
    }

    #[test]
    fn test_track_plan_outcomes_socialize_success() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(UtilityConfig::default());

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.8,
                    rest: 0.8,
                    leisure: 0.7, // Improved from 0.2
                },
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::Socialize,
                    started_at: 50,
                    needs_before: Needs {
                        hunger: 0.8,
                        rest: 0.8,
                        leisure: 0.2,
                    },
                },
            ))
            .id();

        world.run_system_once(track_plan_outcomes_system).unwrap();

        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(
            weights.action_success_count[ActionType::Socialize.as_index()],
            1,
            "Socialize action should be counted as success if leisure improved"
        );
    }

    #[test]
    fn test_penal_labor_prioritizes_work() {
        use crate::layer1::designation::{Designation, DesignationType};
        use crate::layer1::justice::Inmate;
        use crate::layer1::penal::PenalLabor;

        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        // Inmate with PenalLabor
        let inmate = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
                Inmate {
                    sentence_ticks: 100,
                },
                PenalLabor::default(),
            ))
            .id();

        // Work designation available
        world.spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Available tavern (normally attractive)
        world.spawn((
            crate::layer1::social::Tavern::default(),
            GridPosition { x: 1, y: 1 },
        ));

        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(inmate).unwrap();
        assert_eq!(
            action.current,
            ActionType::Work,
            "Penal Labor should prioritize Work over other actions"
        );
        assert!(
            action.current_utility > 1.0,
            "Should have high utility bonus"
        );
    }
}
