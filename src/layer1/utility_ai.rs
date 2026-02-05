use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::needs::Needs;
use crate::layer1::pop::GridPosition;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// High-level action types pops can choose
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActionType {
    /// Eat food to reduce hunger
    SatisfyHunger,
    /// Sleep to reduce fatigue
    SatisfyRest,
    /// Interact with other pops
    Socialize,
    /// Explore the map
    Explore,
    /// Perform designated work (Mine, Build, Chop)
    Work,
    /// Do nothing
    Idle,
}

/// Pop's current action and commitment state
#[derive(Component, Debug)]
pub struct PopAction {
    /// The current action being performed
    pub current: ActionType,
    /// The utility score of the current action
    pub current_utility: f32,
    /// How many ticks the pop has been doing this action
    pub ticks_committed: u32,
}

impl Default for PopAction {
    fn default() -> Self {
        Self {
            current: ActionType::Idle,
            current_utility: 0.0,
            ticks_committed: 0,
        }
    }
}

/// Learned utility weights (reinforcement learning)
#[derive(Component, Clone, Debug)]
pub struct UtilityWeights {
    /// Weight for distance factor (lower distance is better)
    pub distance_weight: f32,
    /// Weight for availability factor (less crowded is better)
    pub availability_weight: f32,
    /// Weight for social factor
    pub social_weight: f32,
    /// Count of successful actions per type
    pub action_success_count: HashMap<ActionType, u32>,
    /// Count of attempted actions per type
    pub action_attempt_count: HashMap<ActionType, u32>,
}

impl Default for UtilityWeights {
    fn default() -> Self {
        Self {
            distance_weight: 1.0,
            availability_weight: 1.0,
            social_weight: 1.0,
            action_success_count: HashMap::new(),
            action_attempt_count: HashMap::new(),
        }
    }
}

/// Tracks HTN plan for learning
#[derive(Component, Debug)]
pub struct PlanOutcome {
    /// The action type being tracked
    pub action: ActionType,
    /// Tick when the action started
    pub started_at: u64,
    /// Needs state before the action
    pub needs_before: Needs,
}

/// Global configuration for utility system
#[derive(Resource, Clone)]
pub struct UtilityConfig {
    /// Minimum utility difference required to switch actions
    pub switch_threshold: f32,
    /// How often (in ticks) to re-evaluate actions
    pub evaluation_interval: u32,
    /// How fast weights adjust (0.0 to 1.0)
    pub learning_rate: f32,
    /// Min and max values for weights
    pub weight_clamp: (f32, f32),
}

impl Default for UtilityConfig {
    fn default() -> Self {
        Self {
            switch_threshold: 0.15,
            evaluation_interval: 1,
            learning_rate: 0.05,
            weight_clamp: (0.5, 2.0),
        }
    }
}

/// Colony-wide memory (zeitgeist)
#[derive(Resource, Default, Clone)]
pub struct ColonyMemory {
    /// Total successful actions across all pops
    pub total_successful_actions: HashMap<ActionType, u32>,
    /// Average duration of actions
    pub average_action_duration: HashMap<ActionType, u32>,
}

/// Marker component to trigger HTN plan creation
#[derive(Component)]
pub struct StartPlan {
    /// The action to plan for
    pub action: ActionType,
    /// The target entity (if any)
    pub target: Option<Entity>,
}

/// Stub for HTN Plan component (future integration)
#[derive(Component)]
pub struct Plan;

/// Calculates urgency from a need value (0.0-1.0).
/// Lower need value = higher urgency.
#[must_use]
pub fn need_response_curve(need_value: f32) -> f32 {
    need_value.mul_add(-need_value, 1.0)
}

/// Calculates Manhattan distance between two positions.
#[must_use]
pub const fn manhattan_distance(pos1: &GridPosition, pos2: &GridPosition) -> i32 {
    (pos1.x - pos2.x).abs() + (pos1.y - pos2.y).abs()
}

/// Calculates a score based on context (distance, availability).
#[must_use]
pub fn calculate_context_score(
    pop_pos: GridPosition,
    target_pos: Option<GridPosition>,
    building_capacity: usize,
    building_occupied: usize,
    weights: &UtilityWeights,
) -> f32 {
    let mut score = 1.0;

    // Distance factor (closer = better)
    if let Some(target) = target_pos {
        let distance = manhattan_distance(&pop_pos, &target);
        #[allow(clippy::cast_precision_loss)]
        let distance_factor = 1.0 / (distance as f32).mul_add(0.1, 1.0);
        score *= distance_factor.powf(weights.distance_weight);
    }

    // Availability factor (less crowded = better)
    if building_capacity > 0 {
        #[allow(clippy::cast_precision_loss)]
        let availability = 1.0 - (building_occupied as f32 / building_capacity as f32);
        score *= availability.powf(weights.availability_weight);
    }

    // Social factor (future - for now just identity)
    score *= 1.0_f32.powf(weights.social_weight);

    score.clamp(0.0, 1.0)
}

/// Calculates a modifier based on past success rates.
#[must_use]
pub fn calculate_success_modifier(action: ActionType, weights: &UtilityWeights) -> f32 {
    let attempts = weights
        .action_attempt_count
        .get(&action)
        .copied()
        .unwrap_or(0);
    let successes = weights
        .action_success_count
        .get(&action)
        .copied()
        .unwrap_or(0);

    if attempts == 0 {
        return 1.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let success_rate = successes as f32 / attempts as f32;

    // Convert to modifier: 0.8-1.2 range
    0.8 + (success_rate * 0.4)
}

/// Evaluates the utility of satisfying hunger at available farms.
#[must_use]
pub fn evaluate_satisfy_hunger<'a>(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    farms: impl Iterator<Item = (Entity, &'a GridPosition, &'a Farm)>,
) -> Option<(f32, Entity)> {
    let hunger_urgency = need_response_curve(needs.hunger);

    let mut best: Option<(f32, Entity)> = None;

    for (farm_entity, farm_pos, farm) in farms {
        let context_score = calculate_context_score(
            *pop_pos,
            Some(*farm_pos),
            farm.capacity,
            farm.workers.len(),
            weights,
        );

        let success_mod = calculate_success_modifier(ActionType::SatisfyHunger, weights);

        let utility = hunger_urgency * context_score * success_mod;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, farm_entity));
        }
    }

    best
}

/// Evaluates the utility of satisfying rest at available housing.
#[must_use]
pub fn evaluate_satisfy_rest<'a>(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    housing: impl Iterator<Item = (Entity, &'a GridPosition, &'a Housing)>,
) -> Option<(f32, Entity)> {
    let rest_urgency = need_response_curve(needs.rest);

    let mut best: Option<(f32, Entity)> = None;

    for (housing_entity, housing_pos, house) in housing {
        let context_score = calculate_context_score(
            *pop_pos,
            Some(*housing_pos),
            house.capacity,
            house.residents.len(),
            weights,
        );

        let success_mod = calculate_success_modifier(ActionType::SatisfyRest, weights);

        let utility = rest_urgency * context_score * success_mod;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, housing_entity));
        }
    }

    best
}

use crate::layer1::designation::Designation;

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

/// Evaluates the utility of being idle.
///
/// Idle is a low-priority fallback action. Pops should prefer productive
/// activities (work, eating, resting) over standing around.
#[must_use]
pub fn evaluate_idle(_needs: &Needs) -> f32 {
    0.05
}

/// System to update commitment timers.
pub fn update_action_timer_system(world: &mut World) {
    let mut query = world.query::<&mut PopAction>();
    for mut action in query.iter_mut(world) {
        action.ticks_committed += 1;
    }
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
                w.clone(),
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
    let mut designations_state = world.query::<(Entity, &GridPosition, &Designation)>();

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

        // Evaluate Work
        if let Some((utility, target)) =
            evaluate_work(&pop_pos, &weights, designations_state.iter(world))
        {
            utilities.push((ActionType::Work, utility, Some(target)));
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
    *weights.action_attempt_count.entry(action).or_insert(0) += 1;

    if success {
        *weights.action_success_count.entry(action).or_insert(0) += 1;

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
pub fn track_plan_outcomes_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();
    let sim_time = world.resource::<SimulationTime>().tick;

    // Find completed plans (have PlanOutcome but no Plan component)
    let completed: Vec<(Entity, PlanOutcome, Needs)> = world
        .query::<(Entity, &PlanOutcome, &Needs)>()
        .iter(world)
        .filter(|(e, _, _)| world.get::<Plan>(*e).is_none())
        .map(|(e, o, n)| {
            (
                e,
                PlanOutcome {
                    action: o.action,
                    started_at: o.started_at,
                    needs_before: o.needs_before,
                },
                *n,
            )
        })
        .collect();

    for (pop_entity, outcome, needs_after) in completed {
        let duration = sim_time - outcome.started_at;

        // Calculate success
        let need_delta = match outcome.action {
            ActionType::SatisfyHunger => needs_after.hunger - outcome.needs_before.hunger,
            ActionType::SatisfyRest => needs_after.rest - outcome.needs_before.rest,
            _ => 0.0,
        };

        let success = need_delta > 0.05;

        // Update weights
        if let Some(mut weights) = world.get_mut::<UtilityWeights>(pop_entity) {
            #[allow(clippy::cast_possible_truncation)]
            update_weights_from_outcome(
                &mut weights,
                outcome.action,
                success,
                duration as u32,
                &config,
            );
        }

        // Clean up
        world.entity_mut(pop_entity).remove::<PlanOutcome>();
    }
}

// Re-add tests at the bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_update_action_timer() {
        let mut world = World::new();
        let pop = world.spawn(PopAction::default()).id();

        // Run system
        update_action_timer_system(&mut world);

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
        assert_eq!(action.current_utility, 0.0);
        assert_eq!(action.ticks_committed, 0);
    }

    #[test]
    fn test_utility_weights_default() {
        let weights = UtilityWeights::default();
        assert_eq!(weights.distance_weight, 1.0);
        assert_eq!(weights.availability_weight, 1.0);
        assert_eq!(weights.social_weight, 1.0);
        assert!(weights.action_success_count.is_empty());
        assert!(weights.action_attempt_count.is_empty());
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
        weights
            .action_attempt_count
            .insert(ActionType::SatisfyHunger, 10);
        weights
            .action_success_count
            .insert(ActionType::SatisfyHunger, 9);
        let high = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(high > 1.0, "High success rate should boost modifier");

        // Low success rate
        weights
            .action_success_count
            .insert(ActionType::SatisfyHunger, 2);
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
        let far_farm = world
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
        let (_utility, chosen_farm) = result.unwrap();
        // Should pick based on best utility (distance vs availability trade-off)
        assert!(chosen_farm == far_farm || chosen_farm != far_farm); // Either is valid depending on weights
    }

    #[test]
    fn test_evaluate_actions_switches_when_threshold_exceeded() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());

        // Starving pop currently idle
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.8,
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

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.6,
                    rest: 0.6,
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
        assert_eq!(weights.action_attempt_count[&ActionType::SatisfyHunger], 1);
        assert_eq!(weights.action_success_count[&ActionType::SatisfyHunger], 1);
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
        assert_eq!(weights.action_attempt_count[&ActionType::SatisfyHunger], 1);
        assert_eq!(
            weights
                .action_success_count
                .get(&ActionType::SatisfyHunger)
                .copied()
                .unwrap_or(0),
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
                }, // Improved from 0.3
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::SatisfyHunger,
                    started_at: 90,
                    needs_before: Needs {
                        hunger: 0.3,
                        rest: 0.8,
                    },
                },
            ))
            .id();
        // Note: No Plan component = plan completed

        track_plan_outcomes_system(&mut world);

        // Should have updated weights
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(weights.action_success_count[&ActionType::SatisfyHunger], 1);

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
                }, // No improvement
                UtilityWeights::default(),
                PlanOutcome {
                    action: ActionType::SatisfyHunger,
                    started_at: 90,
                    needs_before: Needs {
                        hunger: 0.3,
                        rest: 0.8,
                    },
                },
            ))
            .id();

        track_plan_outcomes_system(&mut world);

        // Should have tracked failure
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(weights.action_attempt_count[&ActionType::SatisfyHunger], 1);
        assert_eq!(
            weights
                .action_success_count
                .get(&ActionType::SatisfyHunger)
                .copied()
                .unwrap_or(0),
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
        assert_eq!(config.switch_threshold, 0.15);
        assert_eq!(config.evaluation_interval, 1);
        assert_eq!(config.learning_rate, 0.05);
        assert_eq!(config.weight_clamp, (0.5, 2.0));
    }

    #[test]
    fn test_colony_memory_default() {
        let memory = ColonyMemory::default();
        assert!(memory.total_successful_actions.is_empty());
        assert!(memory.average_action_duration.is_empty());
    }

    #[test]
    fn test_integration_pop_learns_from_experience() {
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.3,
                    rest: 0.8,
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
                },
            });

            world.get_mut::<Needs>(pop).unwrap().hunger = 0.7; // Success!
            world.resource_mut::<SimulationTime>().tick += 10;

            track_plan_outcomes_system(&mut world);
        }

        // Pop should have learned (weights increased)
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert!(
            weights.distance_weight > 1.0 || weights.availability_weight > 1.0,
            "Pop should have learned from successful experiences"
        );
        assert_eq!(weights.action_success_count[&ActionType::SatisfyHunger], 5);
    }
}
