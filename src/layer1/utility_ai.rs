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

use crate::layer1::actions::admin::evaluate_admin;
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
use crate::layer1::admin::Office;
use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
use crate::layer1::chemical::evaluate_consume_chemical;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::farm::Farm;
use crate::layer1::fauna::Fauna;
use crate::layer1::flora::Flora;
use crate::layer1::funeral::{Corpse, Grave};
use crate::layer1::hobby::evaluate_hobby;
use crate::layer1::housing::Housing;
use crate::layer1::husbandry::evaluate_tame;
use crate::layer1::items::Item;
use crate::layer1::justice::{Wanted, evaluate_warden_action};
use crate::layer1::map::GridPosition;
use crate::layer1::medical::Hospital;
use crate::layer1::refining::get_refining_recipe;
use crate::layer1::resources::{ColonyResources, RefiningProgress, ResourceItem};
use crate::layer1::science::Anomaly;
use crate::layer1::social::Tavern;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::{DeferMaintenance, Structure};
use crate::layer1::tech::Library;
use crate::layer1::traits::Trait;
use crate::layer1::utility_eval_types::{
    PopEvalData, PopEvaluationQuery, ScorableCandidate, UtilityAIBuffer, WorldContext,
    evaluate_idle,
};
pub use crate::layer1::utility_types::{
    ActionType, PopAction, StartPlan, UtilityConfig, UtilityWeights, manhattan_distance,
};
use crate::layer1::zone::ZoneGrid;
use bevy_ecs::prelude::*;
use bevy_tasks::ComputeTaskPool;

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
    let Some(factions) = context.factions.as_ref() else {
        return false;
    };
    let Some(member) = data.faction_member.as_ref() else {
        return false;
    };
    let Some(fid) = member.faction_id else {
        return false;
    };
    let Some(faction_data) = factions.get(&fid) else {
        return false;
    };

    faction_data.state == crate::layer1::factions::FactionState::Striking
}

#[allow(clippy::collapsible_if)]
fn evaluate_group_survival(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) {
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

    // Evaluate SeekMedicalCare
    if let Some(health) = data.health {
        if let Some((utility, target)) =
            evaluate_seek_medical_care(pop_pos, &needs, health, &weights, &buffer.hospitals)
        {
            evaluator.consider(ActionType::SeekMedicalCare, utility, Some(target));
        }
    }

    // Evaluate ConsumeChemical
    if let Some((utility, target)) = evaluate_consume_chemical(
        pop_pos,
        &needs,
        &weights,
        data.chemical_state.as_ref(),
        data.stress,
        &buffer.item_entities,
    ) {
        evaluator.consider(ActionType::ConsumeChemical, utility, Some(target));
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

    // Evaluate Admin
    if !is_penal {
        if let Some((utility, target)) = evaluate_admin(pop_pos, &weights, &buffer.offices) {
            evaluator.consider(ActionType::Admin, utility, Some(target));
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
    if let Some((utility, target)) = evaluate_fetch_clothing(
        pop_pos,
        data.insulation,
        context.resources,
        &buffer.stockpiles,
        context.temperature_grid,
    ) {
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
        &buffer.item_entities,
        &buffer.stockpiles,
        context.resources,
        data.carrying,
        data.carrying_item,
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

fn evaluate_group_leisure(evaluator: &mut CandidateEvaluator, data: &PopEvalData) {
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
    data: &PopEvalData,
    context: &WorldContext,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check for Mental Break (Returns early)
    if let Some((action, utility, target)) = evaluate_mental_break(data, buffer) {
        return (action, utility, target);
    }

    // 1b. Check for Memetic Compulsion (Returns early, overrides drafted)
    if let Some((action, utility, target)) =
        crate::layer1::memetic::evaluate_scrawl_memetic_sigil(data, buffer)
    {
        return (action, utility, target);
    }

    // 2. Check for Drafted (Returns early)
    if let Some((action, utility, target)) = evaluate_drafted_behavior(data, buffer) {
        return (action, utility, target);
    }

    // 3. Normal evaluation (undrafted, sane)
    let mut evaluator = CandidateEvaluator::new(evaluate_idle(&data.needs));
    let is_striking = is_pop_striking(data, context);

    evaluate_group_survival(&mut evaluator, data, buffer);
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
    populate_farms(world, &mut buffer.farms, context.cycle);
    populate_housing(world, &mut buffer.housing);
    populate_taverns(world, &mut buffer.taverns);
    populate_libraries(world, &mut buffer.libraries, context.cycle);
    populate_refining(world, &mut buffer.refining, context);
    populate_hospitals(world, &mut buffer.hospitals);
    populate_offices(world, &mut buffer.offices, context.cycle);
}

fn populate_farms(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    let mut farm_query = world.query::<(
        Entity,
        &GridPosition,
        &Farm,
        Option<&ShiftSchedule>,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (entity, pos, farm, schedule, power) in farm_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        if power.is_some_and(|p| !p.active) {
            continue;
        }
        if farm.workers.len() >= farm.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            farm.capacity,
            farm.workers.len(),
        ));
    }
}

fn populate_housing(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut housing_query = world.query::<(Entity, &GridPosition, &Housing)>();
    for (entity, pos, housing) in housing_query.iter(world) {
        if housing.residents.len() >= housing.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            housing.capacity,
            housing.residents.len(),
        ));
    }
}

fn populate_taverns(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut tavern_query = world.query::<(Entity, &GridPosition, &Tavern)>();
    for (entity, pos, tavern) in tavern_query.iter(world) {
        if tavern.visitors.len() >= tavern.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            tavern.capacity,
            tavern.visitors.len(),
        ));
    }
}

fn populate_libraries(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    let mut library_query =
        world.query::<(Entity, &GridPosition, &Library, Option<&ShiftSchedule>)>();
    for (entity, pos, _library, schedule) in library_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(entity, *pos, 5, 0));
    }
}

fn populate_refining(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    context: &WorldContext,
) {
    buffer.clear();
    let mut refine_query = world.query::<(
        Entity,
        &GridPosition,
        &Building,
        &RefiningProgress,
        Option<&ShiftSchedule>,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (entity, pos, building, progress, schedule, power) in refine_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(context.cycle.time_of_day)) {
            continue;
        }
        if power.is_some_and(|p| !p.active) {
            continue;
        }

        // Check recipe affordability (Global check)
        let (can_afford, _, _, _) = get_refining_recipe(building.building_type, context.resources);
        if !can_afford {
            continue;
        }

        let mut candidate = ScorableCandidate::new(entity, *pos);
        candidate.score_bonus = if progress.current > 0.0 { 0.1 } else { 0.0 };
        buffer.push(candidate);
    }
}

fn populate_hospitals(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut hospital_query = world.query::<(
        Entity,
        &GridPosition,
        &Hospital,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (entity, pos, _, power) in hospital_query.iter(world) {
        if power.is_some_and(|p| !p.active) {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(entity, *pos, 10, 0));
    }
}

fn populate_offices(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    let mut office_query =
        world.query::<(Entity, &GridPosition, &Office, Option<&ShiftSchedule>)>();
    for (entity, pos, office, schedule) in office_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        if office.workers.len() >= office.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            office.capacity,
            office.workers.len(),
        ));
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
                    .push(ScorableCandidate::new(entity, *pos));
            }
            DesignationType::Tame => {
                buffer
                    .tame_designations
                    .push(ScorableCandidate::new(entity, *pos));
            }
            _ => {
                buffer
                    .work_designations
                    .push(ScorableCandidate::new(entity, *pos));
            }
        }
    }
}

fn populate_buffer_items_and_misc(world: &mut World, buffer: &mut UtilityAIBuffer) {
    populate_stockpiles(world, &mut buffer.stockpiles);
    populate_items(world, &mut buffer.items);
    populate_generic_items(world, &buffer.stockpiles, &mut buffer.item_entities);
    populate_anomalies(world, &mut buffer.anomalies);
    populate_corpses(world, &mut buffer.corpses);
    populate_graves(world, &mut buffer.graves);
    populate_repair_structures(world, &mut buffer.repair_structures);
    populate_wanted_criminals(world, &mut buffer.wanted_criminals);
}

fn populate_stockpiles(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut stock_query = world.query::<(Entity, &GridPosition, &Stockpile)>();
    for (entity, pos, _) in stock_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_items(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut item_query = world.query::<(Entity, &GridPosition, &ResourceItem)>();
    for (entity, pos, item) in item_query.iter(world) {
        let mut candidate = ScorableCandidate::new(entity, *pos);
        candidate.resource_type = Some(item.resource_type);
        buffer.push(candidate);
    }
}

fn populate_generic_items(
    world: &mut World,
    stockpiles: &[ScorableCandidate],
    buffer: &mut Vec<ScorableCandidate>,
) {
    buffer.clear();
    let mut item_entity_query = world.query::<(Entity, &GridPosition, &Item)>();
    for (entity, pos, item) in item_entity_query.iter(world) {
        // Optimization: Don't haul items that are already at a stockpile
        let at_stockpile = stockpiles.iter().any(|s| s.pos == *pos);
        if at_stockpile {
            continue;
        }

        let mut candidate = ScorableCandidate::new(entity, *pos);
        candidate.item_type = Some(item.item_type.clone());
        buffer.push(candidate);
    }
}

fn populate_anomalies(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut anomaly_query = world.query::<(Entity, &GridPosition, &Anomaly)>();
    for (entity, pos, _) in anomaly_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_corpses(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut corpse_query = world.query::<(Entity, &GridPosition, &Corpse)>();
    for (entity, pos, _) in corpse_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_graves(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut grave_query = world.query::<(Entity, &GridPosition, &Grave)>();
    for (entity, pos, grave) in grave_query.iter(world) {
        if !grave.occupied {
            buffer.push(ScorableCandidate::new(entity, *pos));
        }
    }
}

fn populate_repair_structures(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut struct_query =
        world.query::<(Entity, &GridPosition, &Structure, Option<&DeferMaintenance>)>();
    for (entity, pos, structure, defer) in struct_query.iter(world) {
        if defer.is_some() {
            continue;
        }
        if (structure.current_hp - structure.max_hp).abs() < f32::EPSILON {
            continue;
        }

        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_wanted_criminals(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut wanted_query = world.query::<(Entity, &GridPosition, &Wanted)>();
    for (entity, pos, _) in wanted_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_walls(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut query = world.query::<(Entity, &GridPosition, &Building)>();
    for (entity, pos, building) in query.iter(world) {
        if building.building_type == BuildingType::Wall {
            buffer.push(ScorableCandidate::new(entity, *pos));
        }
    }
}

fn populate_enemies(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut fauna_query = world.query::<(Entity, &GridPosition, &Fauna)>();
    for (entity, pos, _) in fauna_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
    let mut flora_query = world.query::<(Entity, &GridPosition, &Flora)>();
    for (entity, pos, _) in flora_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_all_structures(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut query = world.query::<(Entity, &GridPosition, &Structure)>();
    for (entity, pos, _) in query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn collect_pop_data(world: &mut World, buffer: &mut UtilityAIBuffer, config: &UtilityConfig) {
    buffer.pop_data.clear();
    buffer.pop_data.extend(
        world
            .query_filtered::<PopEvaluationQuery, Without<crate::layer1::cryo::CryoStasis>>()
            .iter(world)
            .filter(|item| {
                item.action.ticks_committed >= config.evaluation_interval
                    && (item.inmate.is_none() || item.penal_labor.is_some())
            })
            .map(PopEvalData::from_query_item),
    );

    // Populate Insulation from Clothing entities
    let mut clothing_query = world.query::<&crate::layer1::items::Clothing>();
    for data in &mut buffer.pop_data {
        if let Some(eq) = data.equipment {
            if let Some(body_entity) = eq.body {
                if let Ok(clothing) = clothing_query.get(world, body_entity) {
                    data.insulation = clothing.insulation;
                }
            }
        }
    }
}

fn run_evaluations(
    buffer: &UtilityAIBuffer,
    context: &WorldContext,
) -> Vec<Option<(ActionType, f32, Option<Entity>)>> {
    let pool = ComputeTaskPool::get();
    let pop_count = buffer.pop_data.len();
    let thread_count = pool.thread_num();
    let chunk_size = (pop_count / thread_count).max(1);

    // Prepare a vector to hold results, sized to match pop_data
    let mut results = vec![None; pop_count];
    let mut rest = results.as_mut_slice();

    pool.scope(|scope| {
        for chunk in buffer.pop_data.chunks(chunk_size) {
            // Split the results slice to get a mutable chunk for this thread
            let (result_chunk, remaining) = rest.split_at_mut(chunk.len());
            rest = remaining;

            scope.spawn(async move {
                for (i, data) in chunk.iter().enumerate() {
                    let result = evaluate_single_pop(buffer, data, context);
                    result_chunk[i] = Some(result);
                }
            });
        }
    });

    results
}

fn apply_evaluation_results(
    world: &mut World,
    pop_data: &[PopEvalData],
    results: &[Option<(ActionType, f32, Option<Entity>)>],
    config: &UtilityConfig,
) {
    for (i, data) in pop_data.iter().enumerate() {
        if let Some((best_action, best_utility, best_target)) = results[i] {
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
    populate_walls(world, &mut buffer.walls);
    populate_enemies(world, &mut buffer.enemies);
    populate_all_structures(world, &mut buffer.all_structures);
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
#[allow(
    clippy::too_many_lines,
    clippy::collapsible_if,
    clippy::type_complexity
)]
pub fn evaluate_actions_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();

    // Use reusable buffer to avoid repeated heap allocations
    let mut buffer = world
        .remove_resource::<UtilityAIBuffer>()
        .unwrap_or_default();

    // 1. Collect Pop Data
    collect_pop_data(world, &mut buffer, &config);

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

    let temp_grid_opt =
        world.remove_resource::<crate::layer1::temperature::TemperatureGrid>();

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
        temperature_grid: temp_grid_opt.as_ref(),
    };

    // 3. Populate Proxies (The Optimization)
    // We clear buffers and populate them once, filtering invalid targets early.
    populate_ai_buffer(world, &mut buffer, &context);

    // 4. Evaluate each pop (Parallel)
    let results = run_evaluations(&buffer, &context);

    // 5. Apply Results
    apply_evaluation_results(world, &buffer.pop_data, &results, &config);

    // 6. Restore Resources
    world.insert_resource(buffer);

    if let Some(zg) = zone_grid_opt {
        world.insert_resource(zg);
    }
    if let Some(tg) = temp_grid_opt {
        world.insert_resource(tg);
    }
    if let Some(f) = factions_res {
        world.insert_resource(f);
    }
}

// Re-add tests at the bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;
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
