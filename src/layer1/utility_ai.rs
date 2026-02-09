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

/// Mathematical functions for utility scoring.
pub mod math;
/// Core types for Utility AI.
pub mod types;

pub use math::*;
pub use types::*;

use crate::layer1::actions::explore::evaluate_explore;
use crate::layer1::actions::fetch_tool::evaluate_fetch_tool;
use crate::layer1::actions::haul::evaluate_haul;
use crate::layer1::actions::hunger::evaluate_satisfy_hunger;
use crate::layer1::actions::idle::evaluate_idle;
use crate::layer1::actions::repair::evaluate_repair;
use crate::layer1::actions::research::evaluate_research;
use crate::layer1::actions::rest::evaluate_satisfy_rest;
use crate::layer1::actions::work::evaluate_work;
use crate::layer1::designation::Designation;
use crate::layer1::farm::Farm;
use crate::layer1::funeral::{Corpse, Grave, evaluate_bury_corpse};
use crate::layer1::housing::Housing;
use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::medical::{Hospital, evaluate_seek_medical_care};
use crate::layer1::needs::Needs;
use crate::layer1::resources::{ColonyResources, ResourceItem};
use crate::layer1::science::Anomaly;
use crate::layer1::social::{Tavern, evaluate_socialize};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::tech::Library;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// System to update commitment timers.
/// Increments the committed-tick counter for every pop's action.
///
/// Uses `par_iter_mut` for parallel processing across entities.
pub fn update_action_timer_system(mut query: Query<&mut PopAction>) {
    query.par_iter_mut().for_each(|mut action| {
        action.ticks_committed += 1;
    });
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

    // Collect pop data
    #[allow(unused_mut)]
    // We break up the query to avoid complex iterator types
    let mut pop_data: Vec<(
        Entity,
        GridPosition,
        Needs,
        UtilityWeights,
        PopAction,
        Option<Equipment>,
    )> = world
        .query::<(
            Entity,
            &GridPosition,
            &Needs,
            &UtilityWeights,
            &PopAction,
            Option<&Equipment>,
        )>()
        .iter(world)
        .filter(|(_, _, _, _, action, _)| action.ticks_committed >= config.evaluation_interval)
        .map(|(e, p, n, w, a, eq)| {
            (
                e,
                *p,
                *n,
                *w,
                PopAction {
                    current: a.current,
                    current_utility: a.current_utility,
                    ticks_committed: a.ticks_committed,
                },
                eq.cloned(),
            )
        })
        .collect();

    // Pre-create query states to avoid allocation in loop
    let mut farms_state = world.query::<(Entity, &GridPosition, &Farm)>();
    let mut housing_state = world.query::<(Entity, &GridPosition, &Housing)>();
    let mut taverns_state = world.query::<(Entity, &GridPosition, &Tavern)>();
    let mut libraries_state = world.query::<(Entity, &GridPosition, &Library)>();
    let mut designations_state = world.query::<(Entity, &GridPosition, &Designation)>();
    let mut items_state = world.query::<(Entity, &GridPosition, &ResourceItem)>();
    let mut stockpiles_state = world.query::<(Entity, &GridPosition, &Stockpile)>();
    let mut anomalies_state = world.query::<(Entity, &GridPosition, &Anomaly)>();
    let mut hospitals_state = world.query::<(Entity, &GridPosition, &Hospital)>();
    let mut corpses_state = world.query::<(Entity, &GridPosition, &Corpse)>();
    let mut graves_state = world.query::<&Grave>();

    // We need Health for medical care evaluation.
    // The main query above only extracted (Entity, &GridPosition, &Needs, &UtilityWeights, &PopAction)
    // We should probably add Health to it, or fetch it.
    // Since Health is optional (maybe?), let's add it to the main query if possible, or get it inside.
    // The pop_data collection above collects into a Vec, effectively decoupling from World.
    // So we can't easily get Health later if we don't collect it.
    // Let's modify the pop_data collection to include Health.

    // But modifying the big query requires modifying the collection logic.
    // Alternatively, we can use `world.get::<Health>(pop_entity)` inside the loop if we didn't collect the data into a detached Vec.
    // But `pop_data` IS a detached Vec. And we are borrowing world mutably in `evaluate_actions_system`.
    // Wait, `evaluate_actions_system` takes `world: &mut World`.
    // The iteration `for (pop_entity, ...)` iterates over the `pop_data` Vec.
    // Inside the loop, we call `evaluate_*` functions passing iterators derived from `world`.
    // We CAN access `world.get::<Health>(pop_entity)` inside the loop?
    // No, `farms_state.iter(world)` borrows world immutably.
    // So we can use `world` immutably inside the loop.

    let resources = world.resource::<ColonyResources>().clone();

    // Evaluate each pop
    for (pop_entity, pop_pos, needs, weights, mut action, equipment_opt) in pop_data {
        // Optimization: Avoid heap allocation (Vec) for utilities.
        // Instead, track the best action found so far in a single pass.

        // Start with Idle as the baseline
        let mut best_action = ActionType::Idle;
        let mut best_utility = evaluate_idle(&needs);
        let mut best_target = None;

        // Helper to update best if we found something better
        let mut check_best = |act, util, tgt| {
            if util > best_utility {
                best_action = act;
                best_utility = util;
                best_target = tgt;
            }
        };

        // Check Health
        let health = world.get::<crate::layer1::health::Health>(pop_entity);

        // Evaluate SatisfyHunger
        if let Some((utility, target)) =
            evaluate_satisfy_hunger(&pop_pos, &needs, &weights, farms_state.iter(world))
        {
            check_best(ActionType::SatisfyHunger, utility, Some(target));
        }

        // Evaluate SatisfyRest
        if let Some((utility, target)) =
            evaluate_satisfy_rest(&pop_pos, &needs, &weights, housing_state.iter(world))
        {
            check_best(ActionType::SatisfyRest, utility, Some(target));
        }

        // Evaluate Socialize
        if let Some((utility, target)) =
            evaluate_socialize(&pop_pos, &needs, &weights, taverns_state.iter(world))
        {
            check_best(ActionType::Socialize, utility, Some(target));
        }

        // Evaluate Work
        if let Some((utility, target)) =
            evaluate_work(&pop_pos, &weights, designations_state.iter(world))
        {
            check_best(ActionType::Work, utility, Some(target));
        }

        // Evaluate FetchTool
        let equipment = equipment_opt.unwrap_or_default();
        if let Some((utility, target)) = evaluate_fetch_tool(
            &pop_pos,
            &equipment,
            &resources,
            stockpiles_state.iter(world),
        ) {
            check_best(ActionType::FetchTool, utility, Some(target));
        }

        // Evaluate Repair
        if let Some((utility, target)) =
            evaluate_repair(&pop_pos, &weights, designations_state.iter(world))
        {
            check_best(ActionType::Repair, utility, Some(target));
        }

        // Evaluate Explore
        if let Some((utility, target)) =
            evaluate_explore(&pop_pos, &weights, anomalies_state.iter(world))
        {
            check_best(ActionType::Explore, utility, Some(target));
        }

        // Evaluate Research
        if let Some((utility, target)) =
            evaluate_research(&pop_pos, &weights, &resources, libraries_state.iter(world))
        {
            check_best(ActionType::Research, utility, Some(target));
        }

        // Evaluate Haul
        if let Some((utility, target)) = evaluate_haul(
            &pop_pos,
            &weights,
            items_state.iter(world),
            stockpiles_state.iter(world),
            &resources,
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
                hospitals_state.iter(world),
            ) {
                check_best(ActionType::SeekMedicalCare, utility, Some(target));
            }
        }

        // Evaluate BuryCorpse
        if let Some((utility, target)) = evaluate_bury_corpse(
            &pop_pos,
            corpses_state.iter(world),
            graves_state.iter(world),
            &weights,
        ) {
            check_best(ActionType::BuryCorpse, utility, Some(target));
        }

        // Switch if best exceeds threshold
        if best_utility > action.current_utility + config.switch_threshold {
            // Update action
            action.current = best_action;
            action.current_utility = best_utility;
            action.ticks_committed = 0;

            // Write back to world
            if let Some(mut pop_action) = world.get_mut::<PopAction>(pop_entity) {
                *pop_action = action;
            }

            // Insert StartPlan marker (for HTN system)
            world.entity_mut(pop_entity).insert(StartPlan {
                action: best_action,
                target: best_target,
            });
        }
    }
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
            ActionType::Work
            | ActionType::Repair
            | ActionType::Socialize
            | ActionType::Explore
            | ActionType::Research
            | ActionType::Haul
            | ActionType::SeekMedicalCare
            | ActionType::BuryCorpse
            | ActionType::FetchTool
            | ActionType::Idle => true,
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
        World::new()
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
    fn test_action_type_variants() {
        let hunger = ActionType::SatisfyHunger;
        let rest = ActionType::SatisfyRest;
        let idle = ActionType::Idle;

        assert_ne!(hunger, rest);
        assert_ne!(hunger, idle);
        assert_eq!(idle, ActionType::Idle);
    }

    #[test]
    fn test_pop_action_default() {
        let action = PopAction::default();
        assert_eq!(action.current, ActionType::Idle);
        assert!((action.current_utility - 0.0).abs() < f32::EPSILON);
        assert_eq!(action.ticks_committed, 0);
    }

    #[test]
    fn test_utility_weights_default() {
        let weights = UtilityWeights::default();
        assert!((weights.distance_weight - 1.0).abs() < f32::EPSILON);
        assert!((weights.availability_weight - 1.0).abs() < f32::EPSILON);
        assert!((weights.social_weight - 1.0).abs() < f32::EPSILON);
        assert!(weights.action_success_count.iter().all(|&x| x == 0));
        assert!(weights.action_attempt_count.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_need_response_curve() {
        // High need value (satisfied) = low urgency
        let urgency_high = need_response_curve(0.9);
        assert!(urgency_high < 0.2, "Satisfied need should have low urgency");

        // Medium need = medium urgency
        let urgency_med = need_response_curve(0.5);
        assert!(urgency_med > 0.6 && urgency_med < 0.8);

        // Low need (critical) = high urgency
        let urgency_low = need_response_curve(0.1);
        assert!(urgency_low > 0.95, "Critical need should have high urgency");

        // Curve should be monotonic (lower need = higher urgency)
        assert!(urgency_low > urgency_med);
        assert!(urgency_med > urgency_high);
    }

    #[test]
    fn test_calculate_context_score_distance() {
        let weights = UtilityWeights::default();

        // Close building (distance 1)
        let close_score = calculate_context_score(
            GridPosition { x: 0, y: 0 },
            Some(GridPosition { x: 1, y: 0 }),
            4, // capacity
            0, // occupied
            &weights,
        );

        // Far building (distance 10)
        let far_score = calculate_context_score(
            GridPosition { x: 0, y: 0 },
            Some(GridPosition { x: 10, y: 0 }),
            4,
            0,
            &weights,
        );

        assert!(
            close_score > far_score,
            "Closer building should have higher score"
        );
    }

    #[test]
    fn test_calculate_context_score_availability() {
        let weights = UtilityWeights::default();
        let pos = GridPosition { x: 0, y: 0 };

        // Empty building
        let empty_score = calculate_context_score(
            pos,
            Some(GridPosition { x: 5, y: 5 }),
            4, // capacity
            0, // occupied
            &weights,
        );

        // Nearly full building
        let full_score = calculate_context_score(
            pos,
            Some(GridPosition { x: 5, y: 5 }),
            4, // capacity
            3, // occupied
            &weights,
        );

        assert!(
            empty_score > full_score,
            "Empty building should have higher score"
        );
    }

    #[test]
    fn test_calculate_success_modifier() {
        let mut weights = UtilityWeights::default();

        // No history = neutral modifier (~1.0)
        let neutral = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(neutral > 0.9 && neutral < 1.1);

        // High success rate
        weights.action_attempt_count[ActionType::SatisfyHunger.as_index()] = 10;
        weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 9;
        let high = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(high > 1.0, "High success rate should boost modifier");

        // Low success rate
        weights.action_success_count[ActionType::SatisfyHunger.as_index()] = 2;
        let low = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(low < 1.0, "Low success rate should reduce modifier");
    }

    #[test]
    fn test_evaluate_satisfy_hunger_finds_best_farm() {
        let mut world = World::new();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            hunger: 0.3,
            rest: 0.8,
            leisure: 0.8,
        };
        let weights = UtilityWeights::default();

        // Close but crowded farm
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 2, y: 0 },
            Farm {
                capacity: 2,
                workers: vec![Entity::from_raw(999)],
            },
        ));

        // Far but empty farm
        let _far_farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 10, y: 0 },
                Farm {
                    capacity: 2,
                    workers: vec![],
                },
            ))
            .id();

        let mut farms = world.query::<(Entity, &GridPosition, &Farm)>();
        let result = evaluate_satisfy_hunger(&pop_pos, &needs, &weights, farms.iter(&world));

        assert!(result.is_some());
    }

    #[test]
    fn test_evaluate_actions_switches_when_threshold_exceeded() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());

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
    fn test_manhattan_distance() {
        let pos1 = GridPosition { x: 0, y: 0 };
        let pos2 = GridPosition { x: 3, y: 4 };

        assert_eq!(manhattan_distance(&pos1, &pos2), 7);

        let pos3 = GridPosition { x: -2, y: 5 };
        assert_eq!(manhattan_distance(&pos1, &pos3), 7);
    }

    #[test]
    fn test_utility_config_default() {
        let config = UtilityConfig::default();
        assert!((config.switch_threshold - 0.15).abs() < f32::EPSILON);
        assert_eq!(config.evaluation_interval, 1);
        assert!((config.learning_rate - 0.05).abs() < f32::EPSILON);
        assert_eq!(config.weight_clamp, (0.5, 2.0));
    }

    #[test]
    fn test_colony_memory_default() {
        let memory = ColonyMemory::default();
        assert!(memory.total_successful_actions.iter().all(|&x| x == 0));
        assert!(memory.average_action_duration.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_integration_pop_learns_from_experience() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());

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
}
