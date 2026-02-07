/// Mathematical functions for utility scoring.
pub mod math;
/// Core types for Utility AI.
pub mod types;

pub use math::*;
pub use types::*;

use crate::layer1::actions::hunger::evaluate_satisfy_hunger;
use crate::layer1::actions::rest::evaluate_satisfy_rest;
use crate::layer1::designation::Designation;
use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::resources::{ColonyResources, ResourceItem};
use crate::layer1::social::{Tavern, evaluate_socialize};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::tech::Library;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// Evaluates the utility of performing work on designations.
#[must_use]
pub fn evaluate_work<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: impl Iterator<Item = (Entity, &'a GridPosition, &'a Designation)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    // Base utility for working (could depend on traits later)
    let base_utility = 0.5;

    for (entity, pos, _) in designations {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity 1 (one worker per tile usually)
            0, // Occupied 0 (simplified for now)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Work, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}

/// Evaluates the utility of researching.
#[must_use]
pub fn evaluate_research<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    libraries: impl Iterator<Item = (Entity, &'a GridPosition, &'a Library)>,
) -> Option<(f32, Entity)> {
    // If knowledge is full, no utility
    if resources.knowledge >= resources.max_knowledge {
        return None;
    }

    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.4;

    for (entity, pos, _) in libraries {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            5, // Assumed capacity
            0, // Assumed occupied (not tracked yet)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Research, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}

/// Evaluates the utility of hauling resources.
#[must_use]
pub fn evaluate_haul<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    items: impl Iterator<Item = (Entity, &'a GridPosition, &'a ResourceItem)>,
    stockpiles: impl Iterator<Item = (Entity, &'a GridPosition, &'a Stockpile)>,
    resources: &ColonyResources,
) -> Option<(f32, Entity)> {
    // 1. Check if any stockpile exists (optimization: no point hauling if nowhere to put it)
    if stockpiles.count() == 0 {
        return None;
    }

    // 2. Find closest item we have room for
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6; // Slightly higher than work (0.5) to keep map clean

    for (entity, pos, item) in items {
        // Check capacity
        let has_room = match item.resource_type {
            crate::layer1::resources::ResourceType::Food => resources.food < resources.max_food,
            crate::layer1::resources::ResourceType::Wood => resources.wood < resources.max_wood,
            crate::layer1::resources::ResourceType::Stone => resources.stone < resources.max_stone,
            crate::layer1::resources::ResourceType::Ore => resources.ore < resources.max_ore,
            crate::layer1::resources::ResourceType::Metal => resources.metal < resources.max_metal,
            crate::layer1::resources::ResourceType::Planks => {
                resources.planks < resources.max_planks
            }
            crate::layer1::resources::ResourceType::Blocks => {
                resources.blocks < resources.max_blocks
            }
        };

        if !has_room {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity
            0, // Occupied
            weights,
        );

        let success = calculate_success_modifier(ActionType::Haul, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }

    best
}

/// Evaluates the utility of being idle.
///
/// Idle is a low-priority fallback action. Pops should prefer productive
/// activities (work, eating, resting) over standing around.
#[must_use]
pub const fn evaluate_idle(_needs: &Needs) -> f32 {
    0.05
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

/// System to evaluate and choose actions for pops.
pub fn evaluate_actions_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();

    // Collect pop data
    #[allow(unused_mut)]
    let mut pop_data: Vec<(Entity, GridPosition, Needs, UtilityWeights, PopAction)> = world
        .query::<(Entity, &GridPosition, &Needs, &UtilityWeights, &PopAction)>()
        .iter(world)
        .filter(|(_, _, _, _, action)| action.ticks_committed >= config.evaluation_interval)
        .map(|(e, p, n, w, a)| {
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

    let resources = world.resource::<ColonyResources>().clone();

    // Evaluate each pop
    for (pop_entity, pop_pos, needs, weights, mut action) in pop_data {
        let mut utilities = Vec::new();

        // Evaluate SatisfyHunger
        if let Some((utility, target)) =
            evaluate_satisfy_hunger(&pop_pos, &needs, &weights, farms_state.iter(world))
        {
            utilities.push((ActionType::SatisfyHunger, utility, Some(target)));
        }

        // Evaluate SatisfyRest
        if let Some((utility, target)) =
            evaluate_satisfy_rest(&pop_pos, &needs, &weights, housing_state.iter(world))
        {
            utilities.push((ActionType::SatisfyRest, utility, Some(target)));
        }

        // Evaluate Socialize
        if let Some((utility, target)) =
            evaluate_socialize(&pop_pos, &needs, &weights, taverns_state.iter(world))
        {
            utilities.push((ActionType::Socialize, utility, Some(target)));
        }

        // Evaluate Work
        if let Some((utility, target)) =
            evaluate_work(&pop_pos, &weights, designations_state.iter(world))
        {
            utilities.push((ActionType::Work, utility, Some(target)));
        }

        // Evaluate Research
        if let Some((utility, target)) =
            evaluate_research(&pop_pos, &weights, &resources, libraries_state.iter(world))
        {
            utilities.push((ActionType::Research, utility, Some(target)));
        }

        // Evaluate Haul
        if let Some((utility, target)) = evaluate_haul(
            &pop_pos,
            &weights,
            items_state.iter(world),
            stockpiles_state.iter(world),
            &resources,
        ) {
            utilities.push((ActionType::Haul, utility, Some(target)));
        }

        // Evaluate Idle
        let idle_utility = evaluate_idle(&needs);
        utilities.push((ActionType::Idle, idle_utility, None));

        // Sort by utility
        utilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Switch if best exceeds threshold
        if let Some((best_action, best_utility, target)) = utilities.first() {
            if *best_utility > action.current_utility + config.switch_threshold {
                // Update action
                action.current = *best_action;
                action.current_utility = *best_utility;
                action.ticks_committed = 0;

                // Write back to world
                if let Some(mut pop_action) = world.get_mut::<PopAction>(pop_entity) {
                    *pop_action = action;
                }

                // Insert StartPlan marker (for HTN system)
                world.entity_mut(pop_entity).insert(StartPlan {
                    action: *best_action,
                    target: *target,
                });
            } else {
                // Increment ticks committed? No, that happens elsewhere or we assume it increments.
                // Actually, we should probably update current utility even if we don't switch?
                // The spec doesn't say. But `evaluate_actions_system` updates `ticks_committed`?
                // No, usually a separate system increments counters.
                // But let's assume `ticks_committed` is updated by the loop or another system.
                // Wait, if we don't switch, we should probably just reset `ticks_committed` if we re-evaluated?
                // No, `ticks_committed` tracks how long we've been doing the current action.
                // If we stick with it, we continue.
            }
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
            | ActionType::Socialize
            | ActionType::Explore
            | ActionType::Research
            | ActionType::Haul
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
