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
//! 3.  **Select Best**: The action with the highest score wins.
//! 4.  **Commit**: The Pop commits to the action for a duration or until a better option appears.
//!
//! ## Key Components
//!
//! *   [`evaluate_actions_system`]: The main loop that runs the decision cycle.
//! *   [`ActionType`]: The enum of all possible behaviors.
//! *   [`UtilityWeights`]: The "memory" of the Pop, adjusting scores based on past success/failure.
//!
//! ## Architecture (ADR 026)
//!
//! The Utility AI system is one of the most computationally expensive parts of the simulation.
//! To maintain high FPS with hundreds of agents, it uses a **Split-Phase Parallel Architecture**:
//!
//! 1.  **Phase 1: Data Gathering (Main Thread)**
//!     *   Collects all world state (Buildings, Items, Designations) into a [`UtilityAIBuffer`].
//!     *   This converts fragmented ECS queries into contiguous memory arrays (SoA layout).
//!     *   See [`crate::layer1::utility_ai_population`].
//!
//! 2.  **Phase 2: Evaluation (Parallel Threads)**
//!     *   The `UtilityAIBuffer` is sliced into chunks and processed by the [`ComputeTaskPool`].
//!     *   Each thread evaluates `evaluate_single_pop` for its chunk of pops.
//!     *   Crucially, this phase represents the world as **Read-Only Context**, avoiding
//!         any need for locks or synchronization.
//!
//! 3.  **Phase 3: Application (Main Thread)**
//!     *   The best actions are collected and written back to the ECS `World`.
//!     *   Only here do we mutate the `PopAction` components.
//!
//! ## How to Add a New Action
//!
//! 1.  **Define the Action**: Add a variant to [`ActionType`] in `utility_types.rs`.
//! 2.  **Create the Evaluator**:
//!     *   Create a new file `src/layer1/actions/my_action.rs`.
//!     *   Implement `evaluate_my_action(pop, needs, targets) -> Option<(f32, Entity)>`.
//!     *   Use [`calculate_context_score`] to handle distance/crowding logic.
//! 3.  **Register Candidates**:
//!     *   Update [`crate::layer1::utility_ai_population::populate_ai_buffer`] to collect
//!         the target entities (e.g., `buffer.my_targets.push(...)`).
//! 4.  **Wire it Up**:
//!     *   Add a call to `evaluate_my_action` in `evaluate_single_pop` (inside this file).
//!     *   Add the execution logic in `src/layer1/execution.rs` (how to actually *do* the task).
//!

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
use crate::layer1::chemical::evaluate_consume_chemical;
use crate::layer1::factions::Factions;
use crate::layer1::hobby::evaluate_hobby;
use crate::layer1::husbandry::evaluate_tame;
use crate::layer1::justice::evaluate_warden_action;
use crate::layer1::resources::ColonyResources;
use crate::layer1::temperature::TemperatureGrid;
use crate::layer1::traits::Trait;
use crate::layer1::utility_ai_population::{collect_pop_data, populate_ai_buffer};
use crate::layer1::utility_eval_types::{
    PopEvalData, UtilityAIBuffer, WorldContext, evaluate_idle,
};
pub use crate::layer1::utility_types::{
    ActionType, PopAction, StartPlan, UtilityConfig, UtilityWeights, manhattan_distance,
};
use crate::layer1::zone::ZoneGrid;
use bevy_ecs::prelude::*;
use bevy_tasks::ComputeTaskPool;

/// Helper struct to manage the lifecycle of resources during utility evaluation.
struct ScopedEvaluationContext {
    buffer: UtilityAIBuffer,
    zone_grid: Option<ZoneGrid>,
    temperature_grid: Option<TemperatureGrid>,
    factions: Option<Factions>,

    // Cloned resources for context
    resources: ColonyResources,
    cycle: crate::layer1::day_night::DayNightCycle,
    taboo: crate::layer1::taboo::TabooState,

    // Config needed for collection/application
    config: UtilityConfig,
}

impl ScopedEvaluationContext {
    fn new(world: &mut World) -> Self {
        let config = world.resource::<UtilityConfig>().clone();

        // 1. Buffer (must be removed before collection if we want to mutate it while world is mutable)
        let mut buffer = world
            .remove_resource::<UtilityAIBuffer>()
            .unwrap_or_default();

        collect_pop_data(world, &mut buffer, &config);

        // 2. Remove other resources
        let zone_grid = world.remove_resource::<ZoneGrid>();
        let temperature_grid = world.remove_resource::<TemperatureGrid>();
        let factions = world.remove_resource::<Factions>();

        // 3. Clone others
        let resources = world.resource::<ColonyResources>().clone();
        let cycle = world
            .resource::<crate::layer1::day_night::DayNightCycle>()
            .clone();
        let taboo = world.resource::<crate::layer1::taboo::TabooState>().clone();

        Self {
            buffer,
            zone_grid,
            temperature_grid,
            factions,
            resources,
            cycle,
            taboo,
            config,
        }
    }

    fn restore(self, world: &mut World) {
        world.insert_resource(self.buffer);
        if let Some(zg) = self.zone_grid {
            world.insert_resource(zg);
        }
        if let Some(tg) = self.temperature_grid {
            world.insert_resource(tg);
        }
        if let Some(f) = self.factions {
            world.insert_resource(f);
        }
    }

    fn populate(&mut self, world: &mut World) {
        // Construct temporary context
        // ZoneGrid fallback logic
        let zone_grid_fallback = ZoneGrid::new(1, 1);
        let zone_grid_ref = self.zone_grid.as_ref().unwrap_or(&zone_grid_fallback);

        let context = WorldContext {
            resources: &self.resources,
            cycle: &self.cycle,
            taboo: &self.taboo,
            factions: self.factions.as_ref().map(|f| &f.map),
            zone_grid: zone_grid_ref,
            temperature_grid: self.temperature_grid.as_ref(),
        };

        populate_ai_buffer(world, &mut self.buffer, &context);
    }

    fn run(&mut self) {
        let zone_grid_fallback = ZoneGrid::new(1, 1);
        let zone_grid_ref = self.zone_grid.as_ref().unwrap_or(&zone_grid_fallback);

        let context = WorldContext {
            resources: &self.resources,
            cycle: &self.cycle,
            taboo: &self.taboo,
            factions: self.factions.as_ref().map(|f| &f.map),
            zone_grid: zone_grid_ref,
            temperature_grid: self.temperature_grid.as_ref(),
        };

        let mut results = std::mem::take(&mut self.buffer.results);
        results.resize(self.buffer.pop_data.len(), None);

        run_evaluations(&self.buffer, &context, &mut results);

        self.buffer.results = results;
    }

    fn apply(self, world: &mut World) {
        apply_evaluation_results(
            world,
            &self.buffer.pop_data,
            &self.buffer.results,
            &self.config,
        );
        self.restore(world);
    }
}

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

    /// Helper to evaluate a result and consider it with penalties/bonuses.
    fn evaluate_and_consider(
        &mut self,
        evaluation: Option<(f32, Entity)>,
        action: ActionType,
        context: &WorldContext,
        bonus: f32,
    ) {
        if let Some((utility, target)) = evaluation {
            let penalty = crate::layer1::taboo::evaluate_taboo_penalty(action, context.taboo);
            self.consider(action, utility + penalty + bonus, Some(target));
        }
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
    context: &WorldContext,
) {
    let pop_pos = data.pos;
    let needs = data.needs;
    let weights = data.weights;

    // Evaluate Hunger
    evaluator.evaluate_and_consider(
        evaluate_satisfy_hunger(pop_pos, &needs, &weights, &buffer.farms),
        ActionType::SatisfyHunger,
        context,
        0.0,
    );

    // Evaluate SatisfyRest
    evaluator.evaluate_and_consider(
        evaluate_satisfy_rest(pop_pos, &needs, &weights, &buffer.housing),
        ActionType::SatisfyRest,
        context,
        0.0,
    );

    // Evaluate SeekMedicalCare
    if let Some(health) = data.health {
        evaluator.evaluate_and_consider(
            evaluate_seek_medical_care(pop_pos, &needs, health, &weights, &buffer.hospitals),
            ActionType::SeekMedicalCare,
            context,
            0.0,
        );
    }

    // Evaluate ConsumeChemical
    evaluator.evaluate_and_consider(
        evaluate_consume_chemical(
            pop_pos,
            &needs,
            &weights,
            data.chemical_state.as_ref(),
            data.stress,
            &buffer.item_entities,
        ),
        ActionType::ConsumeChemical,
        context,
        0.0,
    );
}

fn evaluate_group_social(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    context: &WorldContext,
) {
    let pop_pos = data.pos;
    let needs = data.needs;
    let weights = data.weights;
    let is_penal = data.penal_labor.is_some();

    if is_penal {
        return;
    }

    // Evaluate Socialize
    evaluator.evaluate_and_consider(
        evaluate_socialize(pop_pos, &needs, &weights, &buffer.taverns),
        ActionType::Socialize,
        context,
        0.0,
    );
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
    let work_bonus = if is_penal { 1.0 } else { 0.0 };
    evaluator.evaluate_and_consider(
        evaluate_work(pop_pos, &weights, &buffer.work_designations),
        ActionType::Work,
        context,
        work_bonus,
    );

    // Evaluate Refine
    if !is_penal {
        evaluator.evaluate_and_consider(
            evaluate_refine(pop_pos, &weights, &buffer.refining),
            ActionType::Refine,
            context,
            0.0,
        );
    }

    // Evaluate Farm
    if !is_penal {
        evaluator.evaluate_and_consider(
            evaluate_farm(pop_pos, &weights, &buffer.farms),
            ActionType::Farm,
            context,
            0.0,
        );
    }

    // Evaluate Admin
    if !is_penal {
        evaluator.evaluate_and_consider(
            evaluate_admin(pop_pos, &weights, &buffer.offices),
            ActionType::Admin,
            context,
            0.0,
        );
    }

    // Evaluate Research
    if !is_penal && !is_feral {
        evaluator.evaluate_and_consider(
            evaluate_research(pop_pos, &weights, context.resources, &buffer.libraries),
            ActionType::Research,
            context,
            0.0,
        );
    }

    // Evaluate Tame
    evaluator.evaluate_and_consider(
        evaluate_tame(&pop_pos, &weights, &buffer.tame_designations),
        ActionType::Tame,
        context,
        0.0,
    );

    // Evaluate Warden
    if !is_penal {
        evaluator.evaluate_and_consider(
            evaluate_warden_action(&pop_pos, &buffer.wanted_criminals, context.zone_grid),
            ActionType::Warden,
            context,
            0.0,
        );
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
    evaluator.evaluate_and_consider(
        evaluate_fetch_tool(pop_pos, &equipment, context.resources, &buffer.stockpiles),
        ActionType::FetchTool,
        context,
        0.0,
    );

    // Evaluate FetchClothing
    evaluator.evaluate_and_consider(
        evaluate_fetch_clothing(
            pop_pos,
            data.insulation,
            context.resources,
            &buffer.stockpiles,
            context.temperature_grid,
        ),
        ActionType::FetchClothing,
        context,
        0.0,
    );

    // Evaluate Repair
    evaluator.evaluate_and_consider(
        evaluate_repair(
            pop_pos,
            &weights,
            &buffer.repair_designations,
            &buffer.repair_structures,
        ),
        ActionType::Repair,
        context,
        0.0,
    );

    // Evaluate Haul
    evaluator.evaluate_and_consider(
        evaluate_haul(
            pop_pos,
            &weights,
            &buffer.items,
            &buffer.item_entities,
            &buffer.stockpiles,
            context.resources,
            data.carrying,
            data.carrying_item,
        ),
        ActionType::Haul,
        context,
        0.0,
    );

    // Evaluate BuryCorpse
    evaluator.evaluate_and_consider(
        evaluate_bury_corpse(pop_pos, &buffer.corpses, &buffer.graves, &weights),
        ActionType::BuryCorpse,
        context,
        0.0,
    );
}

fn evaluate_group_exploration(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    context: &WorldContext,
    is_striking: bool,
) {
    let pop_pos = data.pos;
    let weights = data.weights;
    let is_penal = data.penal_labor.is_some();

    if is_striking || is_penal {
        return;
    }

    // Evaluate Explore
    evaluator.evaluate_and_consider(
        evaluate_explore(pop_pos, &weights, &buffer.anomalies),
        ActionType::Explore,
        context,
        0.0,
    );
}

fn evaluate_group_leisure(
    evaluator: &mut CandidateEvaluator,
    data: &PopEvalData,
    context: &WorldContext,
) {
    if let Some(hobby_type) = data.hobby_type {
        let utility = evaluate_hobby(data, hobby_type);
        let penalty =
            crate::layer1::taboo::evaluate_taboo_penalty(ActionType::Hobby, context.taboo);
        evaluator.consider(ActionType::Hobby, utility + penalty, None);
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

    evaluate_group_survival(&mut evaluator, data, buffer, context);
    evaluate_group_social(&mut evaluator, data, buffer, context);
    evaluate_group_leisure(&mut evaluator, data, context);
    evaluate_group_work(&mut evaluator, data, buffer, context, is_striking);
    evaluate_group_logistics(&mut evaluator, data, buffer, context, is_striking);
    evaluate_group_exploration(&mut evaluator, data, buffer, context, is_striking);

    evaluator.result()
}

fn run_evaluations(
    buffer: &UtilityAIBuffer,
    context: &WorldContext,
    results: &mut [Option<(ActionType, f32, Option<Entity>)>],
) {
    let pool = ComputeTaskPool::get();
    let pop_count = buffer.pop_data.len();
    let thread_count = pool.thread_num();
    let chunk_size = (pop_count / thread_count).max(1);

    // Results slice is already provided and resized
    let mut rest = results;

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
    let mut ctx = ScopedEvaluationContext::new(world);

    if ctx.buffer.pop_data.is_empty() {
        ctx.restore(world);
        return;
    }

    ctx.populate(world);
    ctx.run();
    ctx.apply(world);
}

// Re-add tests at the bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::farm::Farm;
    use crate::layer1::map::GridPosition;
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
        crate::setup::init_task_pools();
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
                    hygiene: 0.8,
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
        crate::setup::init_task_pools();
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
                    hygiene: 0.6,
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

        crate::setup::init_task_pools();
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
