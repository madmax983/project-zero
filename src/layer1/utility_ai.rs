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
//!

use crate::layer1::actions::explore::evaluate_explore;
use crate::layer1::actions::farm::evaluate_farm;
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
use crate::layer1::combat::Drafted;
use crate::layer1::designation::Designation;
use crate::layer1::farm::Farm;
use crate::layer1::funeral::{Corpse, Grave};
use crate::layer1::housing::Housing;
use crate::layer1::husbandry::evaluate_tame;
use crate::layer1::items::Equipment;
use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::medical::Hospital;
use crate::layer1::needs::Needs;
use crate::layer1::resources::{ColonyResources, ResourceItem};
use crate::layer1::science::Anomaly;
use crate::layer1::social::Tavern;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::{DeferMaintenance, Structure};
use crate::layer1::tech::Library;
use crate::layer1::unrest::MentalState;
pub use crate::layer1::utility_types::{
    ActionType, Plan, PlanOutcome, PopAction, PopEvalData, StartPlan, UtilityAIBuffer,
    UtilityConfig, UtilityWeights, WorldContext, evaluate_idle, manhattan_distance,
};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryState;

/// Encapsulates all `QueryStates` used to find action candidates.
pub struct CandidateQueries {
    /// Query for farms.
    pub farms: QueryState<(
        Entity,
        &'static GridPosition,
        &'static Farm,
        Option<&'static crate::layer1::building::ShiftSchedule>,
    )>,
    /// Query for refining buildings.
    pub refining: QueryState<(
        Entity,
        &'static GridPosition,
        &'static crate::layer1::building::Building,
        &'static crate::layer1::resources::RefiningProgress,
        Option<&'static crate::layer1::building::ShiftSchedule>,
    )>,
    /// Query for housing.
    pub housing: QueryState<(Entity, &'static GridPosition, &'static Housing)>,
    /// Query for taverns.
    pub taverns: QueryState<(Entity, &'static GridPosition, &'static Tavern)>,
    /// Query for libraries.
    pub libraries: QueryState<(
        Entity,
        &'static GridPosition,
        &'static Library,
        Option<&'static crate::layer1::building::ShiftSchedule>,
    )>,
    /// Query for designations (work targets).
    pub designations: QueryState<(Entity, &'static GridPosition, &'static Designation)>,
    /// Query for loose items (hauling targets).
    pub items: QueryState<(Entity, &'static GridPosition, &'static ResourceItem)>,
    /// Query for stockpiles.
    pub stockpiles: QueryState<(Entity, &'static GridPosition, &'static Stockpile)>,
    /// Query for anomalies (exploration targets).
    pub anomalies: QueryState<(Entity, &'static GridPosition, &'static Anomaly)>,
    /// Query for hospitals.
    pub hospitals: QueryState<(Entity, &'static GridPosition, &'static Hospital)>,
    /// Query for corpses.
    pub corpses: QueryState<(Entity, &'static GridPosition, &'static Corpse)>,
    /// Query for graves.
    pub graves: QueryState<&'static Grave>,
    /// Query for structures (repair targets).
    pub structures: QueryState<(
        Entity,
        &'static GridPosition,
        &'static Structure,
        Option<&'static DeferMaintenance>,
    )>,
}

impl CandidateQueries {
    /// Initializes all queries from the world.
    pub fn new(world: &mut World) -> Self {
        Self {
            farms: world.query(),
            refining: world.query(),
            housing: world.query(),
            taverns: world.query(),
            libraries: world.query(),
            designations: world.query(),
            items: world.query(),
            stockpiles: world.query(),
            anomalies: world.query(),
            hospitals: world.query(),
            corpses: world.query(),
            graves: world.query(),
            structures: world.query(),
        }
    }
}

/// System to update commitment timers.
/// Increments the committed-tick counter for every pop's action.
///
/// Uses `par_iter_mut` for parallel processing across entities.
pub fn update_action_timer_system(mut query: Query<&mut PopAction>) {
    query.par_iter_mut().for_each(|mut action| {
        action.ticks_committed += 1;
    });
}

/// Helper function to evaluate all potential actions for a single Pop.
///
/// Returns the best `(ActionType, Utility, Target)`.
#[allow(clippy::too_many_lines, clippy::collapsible_if)]
pub fn evaluate_single_pop(
    queries: &mut CandidateQueries,
    world: &mut World,
    data: &PopEvalData,
    context: &WorldContext,
) -> (ActionType, f32, Option<Entity>) {
    let pop_entity = data.entity;
    let pop_pos = data.pos;
    let needs = data.needs;
    let weights = data.weights;
    let equipment_opt = data.equipment;

    // Start with Idle as the baseline
    let mut best_action = ActionType::Idle;
    let mut best_utility = evaluate_idle(&needs);
    let mut best_target = None;

    // Helper closure to update best
    let mut check_best = |act: ActionType, util: f32, tgt: Option<Entity>| {
        if util > best_utility {
            best_action = act;
            best_utility = util;
            best_target = tgt;
        }
    };

    // 1. Check for Mental Break
    // We must pass world because evaluate_mental_break constructs its own queries.
    if let Some((action, utility, target)) = evaluate_mental_break(data, world) {
        return (action, utility, target);
    }

    // 2. Check for Drafted
    if let Some((action, utility, target)) = evaluate_drafted_behavior(data, world) {
        return (action, utility, target);
    }

    // 3. Normal evaluation (undrafted, sane)

    // Evaluate Hunger
    if let Some((utility, target)) = evaluate_satisfy_hunger(
        &pop_pos,
        &needs,
        &weights,
        queries.farms.iter(world).map(|(e, p, f, _)| (e, p, f)),
    ) {
        check_best(ActionType::SatisfyHunger, utility, Some(target));
    }

    // Evaluate Work
    let is_striking = context.factions.as_ref().is_some_and(|map| {
        data.faction_member.as_ref().is_some_and(|member| {
            member.faction_id.is_some_and(|fid| {
                map.get(&fid).is_some_and(|data| {
                    data.state == crate::layer1::factions::FactionState::Striking
                })
            })
        })
    });

    if !is_striking {
        if let Some((utility, target)) =
            evaluate_work(&pop_pos, &weights, queries.designations.iter(world))
        {
            let penalty =
                crate::layer1::taboo::evaluate_taboo_penalty(ActionType::Work, context.taboo);
            check_best(ActionType::Work, utility + penalty, Some(target));
        }
    }

    // Check Health
    // Safety: world.get borrows immutable world, which is allowed as long as we don't mutate.
    // evaluate_mental_break/drafted took &mut World but returned, so mutable borrow ended.
    let health = world.get::<crate::layer1::health::Health>(pop_entity);

    // Evaluate SatisfyRest
    if let Some((utility, target)) =
        evaluate_satisfy_rest(&pop_pos, &needs, &weights, queries.housing.iter(world))
    {
        check_best(ActionType::SatisfyRest, utility, Some(target));
    }

    // Evaluate Socialize
    if let Some((utility, target)) =
        evaluate_socialize(&pop_pos, &needs, &weights, queries.taverns.iter(world))
    {
        check_best(ActionType::Socialize, utility, Some(target));
    }

    // Evaluate Refine
    if let Some((utility, target)) = evaluate_refine(
        &pop_pos,
        &weights,
        context.resources,
        context.cycle,
        queries.refining.iter(world),
    ) {
        check_best(ActionType::Refine, utility, Some(target));
    }

    // Evaluate Farm
    if let Some((utility, target)) =
        evaluate_farm(&pop_pos, &weights, context.cycle, queries.farms.iter(world))
    {
        check_best(ActionType::Farm, utility, Some(target));
    }

    // Evaluate FetchTool
    let equipment = equipment_opt.unwrap_or_default();
    if let Some((utility, target)) = evaluate_fetch_tool(
        &pop_pos,
        &equipment,
        context.resources,
        queries.stockpiles.iter(world),
    ) {
        check_best(ActionType::FetchTool, utility, Some(target));
    }

    // Evaluate Repair
    if let Some((utility, target)) = evaluate_repair(
        &pop_pos,
        &weights,
        queries.designations.iter(world),
        queries.structures.iter(world),
    ) {
        check_best(ActionType::Repair, utility, Some(target));
    }

    // Evaluate Explore
    if let Some((utility, target)) =
        evaluate_explore(&pop_pos, &weights, queries.anomalies.iter(world))
    {
        check_best(ActionType::Explore, utility, Some(target));
    }

    // Evaluate Research
    if let Some((utility, target)) = evaluate_research(
        &pop_pos,
        &weights,
        context.resources,
        context.cycle,
        queries.libraries.iter(world),
    ) {
        check_best(ActionType::Research, utility, Some(target));
    }

    // Evaluate Haul
    if let Some((utility, target)) = evaluate_haul(
        &pop_pos,
        &weights,
        queries.items.iter(world),
        queries.stockpiles.iter(world),
        context.resources,
    ) {
        check_best(ActionType::Haul, utility, Some(target));
    }

    // Evaluate SeekMedicalCare
    if let Some(health) = health {
        if let Some((utility, target)) = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            health,
            &weights,
            queries.hospitals.iter(world),
        ) {
            check_best(ActionType::SeekMedicalCare, utility, Some(target));
        }
    }

    // Evaluate BuryCorpse
    if let Some((utility, target)) = evaluate_bury_corpse(
        &pop_pos,
        queries.corpses.iter(world),
        queries.graves.iter(world),
        &weights,
    ) {
        check_best(ActionType::BuryCorpse, utility, Some(target));
    }

    // Evaluate Tame
    if let Some((utility, target)) =
        evaluate_tame(&pop_pos, &weights, queries.designations.iter(world))
    {
        check_best(ActionType::Tame, utility, Some(target));
    }

    (best_action, best_utility, best_target)
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
///     into efficient query iterators.
/// 3.  **Evaluate Candidates**:
///     For each Pop, it calls every `evaluate_*` function:
///     *   [`evaluate_satisfy_hunger`]
///     *   [`evaluate_work`]
///     *   [`evaluate_haul`]
///     *   ...and so on.
/// 4.  **Winner Takes All**: Tracks the single best `(Utility, Action, Target)` tuple.
/// 5.  **Switch**: If the best new utility > current utility + threshold, the Pop switches tasks.
///     *   Updates [`PopAction`].
///     *   Inserts [`StartPlan`] to trigger HTN planning (if applicable).
///
/// # Performance Note
/// This system avoids per-Pop heap allocations by using a single-pass "best so far"
/// tracker instead of collecting a `Vec<ActionCandidate>`.
#[allow(clippy::too_many_lines, clippy::collapsible_if)]
pub fn evaluate_actions_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();

    // Use reusable buffer to avoid repeated heap allocations
    let mut buffer = world
        .remove_resource::<UtilityAIBuffer>()
        .unwrap_or_default();

    buffer.pop_data.clear();

    // Collect pop data into buffer
    buffer.pop_data.extend(
        world
            .query::<(
                Entity,
                &GridPosition,
                &Needs,
                &UtilityWeights,
                &PopAction,
                Option<&Equipment>,
                Option<&MentalState>,
                Option<&Drafted>,
                Option<&Inmate>,
                Option<&crate::layer1::factions::FactionMember>,
            )>()
            .iter(world)
            .filter(|(_, _, _, _, action, _, _, _, inmate, _)| {
                action.ticks_committed >= config.evaluation_interval && inmate.is_none()
            })
            .map(|(e, p, n, w, a, eq, m, d, _, fm)| PopEvalData {
                entity: e,
                pos: *p,
                needs: *n,
                weights: *w,
                action: *a,
                equipment: eq.copied(),
                mental_state: m.copied(),
                drafted: d.copied(),
                faction_member: fm.cloned(),
            }),
    );

    // Initialize Queries
    let mut queries = CandidateQueries::new(world);

    // Initialize Context
    let resources = world.resource::<ColonyResources>().clone();
    let cycle = world
        .resource::<crate::layer1::day_night::DayNightCycle>()
        .clone();
    let taboo = world.resource::<crate::layer1::taboo::TabooState>().clone();
    let factions_data = world
        .get_resource::<crate::layer1::factions::Factions>()
        .map(|f| f.map.clone());

    let context = WorldContext {
        resources: &resources,
        cycle: &cycle,
        taboo: &taboo,
        factions: factions_data.as_ref(),
    };

    // Evaluate each pop
    for data in &buffer.pop_data {
        let (best_action, best_utility, best_target) =
            evaluate_single_pop(&mut queries, world, data, &context);

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
}

/// Updates utility weights based on action outcome.
pub fn update_weights_from_outcome(
    weights: &mut UtilityWeights,
    action: ActionType,
    success: bool,
    duration: u32,
    config: &UtilityConfig,
) {
    // Track attempt
    let idx = action.as_index();
    weights.action_attempt_count[idx] += 1;

    if success {
        weights.action_success_count[idx] += 1;

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
    mut completed: Query<(Entity, &PlanOutcome, &Needs, &mut UtilityWeights), Without<Plan>>,
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
            | ActionType::Slaughter => true,
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
}
