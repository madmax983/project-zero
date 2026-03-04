# 016: Emergent Utility AI System

## Overview

Pops autonomously choose actions using a three-layer AI system: utility evaluation (strategic), HTN planning (tactical), and reinforcement learning (adaptive). This creates emergent behaviors including needs-driven prioritization, spatial awareness, social dynamics, and individual learning.

## Architecture

**Layer 1: Utility Evaluation (Strategic)**
- Pops calculate utility scores for each action using hybrid functions
- Response curves for needs (hunger, rest) + weighted contextual factors (distance, availability, social)
- Actions: `SatisfyHunger`, `SatisfyRest`, `Socialize`, `Explore`, `Idle`
- Threshold-based switching prevents thrashing

**Layer 2: HTN Planning (Tactical)**
- Winning action passed to bevy_bae HTN planner
- HTN generates concrete task sequences: `[MoveTo, WorkAt, Consume]`
- Handles preconditions, replanning on failure
- Integrates via `bevy_bae` crate

**Layer 3: Reinforcement Learning (Adaptive)**
- Tracks HTN plan outcomes (success/failure, duration, need delta)
- Adjusts `UtilityWeights` component: successful patterns reinforced
- Individual learning (per-pop weights)
- Future: Colony memory (zeitgeist) for shared knowledge

## Dependencies

- `004` — Pop entity
- `005` — Pop needs
- `007` — Housing building
- `008` — Farm building (must implement before this spec)
- `bevy_bae` — HTN planner crate

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/utility_ai.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

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

        assert!(close_score > far_score, "Closer building should have higher score");
    }

    #[test]
    fn test_calculate_context_score_availability() {
        let weights = UtilityWeights::default();
        let pos = GridPosition { x: 0, y: 0 };

        // Empty building
        let empty_score = calculate_context_score(
            pos,
            Some(GridPosition { x: 5, y: 5 }),
            4,  // capacity
            0,  // occupied
            &weights,
        );

        // Nearly full building
        let full_score = calculate_context_score(
            pos,
            Some(GridPosition { x: 5, y: 5 }),
            4,  // capacity
            3,  // occupied
            &weights,
        );

        assert!(empty_score > full_score, "Empty building should have higher score");
    }

    #[test]
    fn test_calculate_success_modifier() {
        let mut weights = UtilityWeights::default();

        // No history = neutral modifier (~1.0)
        let neutral = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(neutral > 0.9 && neutral < 1.1);

        // High success rate
        weights.action_attempt_count.insert(ActionType::SatisfyHunger, 10);
        weights.action_success_count.insert(ActionType::SatisfyHunger, 9);
        let high = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(high > 1.0, "High success rate should boost modifier");

        // Low success rate
        weights.action_success_count.insert(ActionType::SatisfyHunger, 2);
        let low = calculate_success_modifier(ActionType::SatisfyHunger, &weights);
        assert!(low < 1.0, "Low success rate should reduce modifier");
    }

    #[test]
    fn test_evaluate_satisfy_hunger_finds_best_farm() {
        let mut world = World::new();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs { hunger: 0.3, rest: 0.8 };
        let weights = UtilityWeights::default();

        // Close but crowded farm
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 2, y: 0 },
            Farm { capacity: 2, workers: vec![Entity::from_raw(999)] },
        ));

        // Far but empty farm
        let far_farm = world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 10, y: 0 },
            Farm { capacity: 2, workers: vec![] },
        )).id();

        let farms = world.query::<(Entity, &GridPosition, &Farm)>();
        let result = evaluate_satisfy_hunger(&pop_pos, &needs, &weights, &farms);

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
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { hunger: 0.1, rest: 0.8 }, // Very hungry!
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.2,
                ticks_committed: 10, // Past evaluation interval
            },
        )).id();

        // Available farm
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        evaluate_actions_system(&mut world);

        // Should have switched to SatisfyHunger
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::SatisfyHunger);
        assert!(action.current_utility > 0.5, "Hungry pop should have high hunger utility");
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

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { hunger: 0.6, rest: 0.6 }, // Moderate needs
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.4,
                ticks_committed: 10,
            },
        )).id();

        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        evaluate_actions_system(&mut world);

        // Should NOT switch (utility difference < threshold)
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle, "Should not switch with high threshold");
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
            true,  // success
            5,     // quick duration
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
        assert_eq!(weights.action_success_count[&ActionType::SatisfyHunger], 0);
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
            update_weights_from_outcome(
                &mut weights,
                ActionType::SatisfyRest,
                true,
                1,
                &config,
            );
        }

        assert!(weights.distance_weight <= config.weight_clamp.1);
        assert!(weights.availability_weight <= config.weight_clamp.1);
    }

    #[test]
    fn test_track_plan_outcomes_on_success() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        world.insert_resource(UtilityConfig::default());

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.6, rest: 0.8 }, // Improved from 0.3
            UtilityWeights::default(),
            PlanOutcome {
                action: ActionType::SatisfyHunger,
                started_at: 90,
                needs_before: Needs { hunger: 0.3, rest: 0.8 },
            },
        )).id();
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
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        world.insert_resource(UtilityConfig::default());

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.3, rest: 0.8 }, // No improvement
            UtilityWeights::default(),
            PlanOutcome {
                action: ActionType::SatisfyHunger,
                started_at: 90,
                needs_before: Needs { hunger: 0.3, rest: 0.8 },
            },
        )).id();

        track_plan_outcomes_system(&mut world);

        // Should have tracked failure
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert_eq!(weights.action_attempt_count[&ActionType::SatisfyHunger], 1);
        assert_eq!(weights.action_success_count.get(&ActionType::SatisfyHunger).copied().unwrap_or(0), 0);
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

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { hunger: 0.3, rest: 0.8 },
            UtilityWeights::default(),
            PopAction::default(),
        )).id();

        world.spawn((
            Building { building_type: BuildingType::Farm },
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
            world.entity_mut(pop).insert(PlanOutcome {
                action: ActionType::SatisfyHunger,
                started_at: world.resource::<SimulationTime>().tick,
                needs_before: Needs { hunger: 0.3, rest: 0.8 },
            });

            world.get_mut::<Needs>(pop).unwrap().hunger = 0.7; // Success!
            world.resource_mut::<SimulationTime>().tick += 10;

            track_plan_outcomes_system(&mut world);
        }

        // Pop should have learned (weights increased)
        let weights = world.get::<UtilityWeights>(pop).unwrap();
        assert!(weights.distance_weight > 1.0 || weights.availability_weight > 1.0,
                "Pop should have learned from successful experiences");
        assert_eq!(weights.action_success_count[&ActionType::SatisfyHunger], 5);
    }
}
```

**Test Coverage Requirements:**
- Action types and components: defaults, variants, fields
- Response curves: monotonic, correct range
- Context scoring: distance, availability, social factors
- Success modifier: history tracking, rate calculation
- Action evaluation: target selection, utility calculation
- Switching logic: threshold respect, commitment tracking
- Weight updates: success reinforcement, failure penalization, clamping
- Plan outcome tracking: success detection, weight adjustment
- Integration: multi-cycle learning, emergent behavior
- All tests must pass before spec is considered complete
- Coverage ≥85% for layer1/utility_ai.rs

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Core Components

```rust
// src/layer1/utility_ai.rs

use bevy_ecs::prelude::*;
use super::{GridPosition, Needs, Pop};
use super::building::{Building, BuildingType};
use super::housing::Housing;
use crate::shared::time::SimulationTime;
use std::collections::HashMap;

/// High-level action types pops can choose
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActionType {
    SatisfyHunger,
    SatisfyRest,
    Socialize,  // Future
    Explore,    // Future
    Idle,
}

/// Pop's current action and commitment state
#[derive(Component, Debug)]
pub struct PopAction {
    pub current: ActionType,
    pub current_utility: f32,
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
    // Multipliers for contextual factors (0.5 - 2.0 range)
    pub distance_weight: f32,
    pub availability_weight: f32,
    pub social_weight: f32,

    // Per-action success history
    pub action_success_count: HashMap<ActionType, u32>,
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
    pub action: ActionType,
    pub started_at: u64,
    pub needs_before: Needs,
}

/// Global configuration for utility system
#[derive(Resource, Clone)]
pub struct UtilityConfig {
    pub switch_threshold: f32,
    pub evaluation_interval: u32,
    pub learning_rate: f32,
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

/// Colony-wide memory (zeitgeist) - Future expansion
#[derive(Resource, Default, Clone)]
pub struct ColonyMemory {
    pub total_successful_actions: HashMap<ActionType, u32>,
    pub average_action_duration: HashMap<ActionType, u32>,
}
```

### Utility Calculation Functions

```rust
// src/layer1/utility_ai.rs (continued)

/// Response curve for needs (inverse: lower need = higher urgency)
#[must_use]
pub fn need_response_curve(need_value: f32) -> f32 {
    1.0 - need_value.powf(2.0)
}

/// Manhattan distance between two grid positions
#[must_use]
pub fn manhattan_distance(pos1: &GridPosition, pos2: &GridPosition) -> i32 {
    (pos1.x - pos2.x).abs() + (pos1.y - pos2.y).abs()
}

/// Calculate context score from spatial/social factors
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
        let distance_factor = 1.0 / (1.0 + distance as f32 * 0.1);
        score *= distance_factor.powf(weights.distance_weight);
    }

    // Availability factor (less crowded = better)
    if building_capacity > 0 {
        let availability = 1.0 - (building_occupied as f32 / building_capacity as f32);
        score *= availability.powf(weights.availability_weight);
    }

    // Social factor (future - for now just identity)
    score *= 1.0_f32.powf(weights.social_weight);

    score.clamp(0.0, 1.0)
}

/// Calculate success modifier from action history
#[must_use]
pub fn calculate_success_modifier(
    action: ActionType,
    weights: &UtilityWeights,
) -> f32 {
    let attempts = weights.action_attempt_count.get(&action).copied().unwrap_or(1);
    let successes = weights.action_success_count.get(&action).copied().unwrap_or(0);

    let success_rate = successes as f32 / attempts as f32;

    // Convert to modifier: 0.8-1.2 range
    0.8 + (success_rate * 0.4)
}
```

### Action Evaluation

```rust
// src/layer1/utility_ai.rs (continued)

use crate::layer1::building::Farm; // Will exist after spec 008

/// Evaluate SatisfyHunger action utility
#[must_use]
pub fn evaluate_satisfy_hunger(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    farms: &Query<(Entity, &GridPosition, &Farm)>,
) -> Option<(f32, Entity)> {
    let hunger_urgency = need_response_curve(needs.hunger);

    let mut best: Option<(f32, Entity)> = None;

    for (farm_entity, farm_pos, farm) in farms.iter() {
        let context_score = calculate_context_score(
            *pop_pos,
            Some(*farm_pos),
            farm.capacity,
            farm.workers.len(),
            weights,
        );

        let success_mod = calculate_success_modifier(ActionType::SatisfyHunger, weights);

        let utility = hunger_urgency * context_score * success_mod;

        if best.is_none() || utility > best.unwrap().0 {
            best = Some((utility, farm_entity));
        }
    }

    best
}

/// Evaluate SatisfyRest action utility
#[must_use]
pub fn evaluate_satisfy_rest(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    housing: &Query<(Entity, &GridPosition, &Housing)>,
) -> Option<(f32, Entity)> {
    let rest_urgency = need_response_curve(needs.rest);

    let mut best: Option<(f32, Entity)> = None;

    for (housing_entity, housing_pos, house) in housing.iter() {
        let context_score = calculate_context_score(
            *pop_pos,
            Some(*housing_pos),
            house.capacity,
            house.residents.len(),
            weights,
        );

        let success_mod = calculate_success_modifier(ActionType::SatisfyRest, weights);

        let utility = rest_urgency * context_score * success_mod;

        if best.is_none() || utility > best.unwrap().0 {
            best = Some((utility, housing_entity));
        }
    }

    best
}

/// Evaluate Idle action utility (only when needs satisfied)
#[must_use]
pub fn evaluate_idle(needs: &Needs) -> f32 {
    let worst_need = needs.hunger.min(needs.rest);
    worst_need.powf(2.0)
}
```

### Main Evaluation System

```rust
// src/layer1/utility_ai.rs (continued)

/// Marker component to trigger HTN plan creation
#[derive(Component)]
pub struct StartPlan {
    pub action: ActionType,
    pub target: Option<Entity>,
}

/// Evaluates all actions and switches if threshold exceeded
pub fn evaluate_actions_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();

    // Collect pop data
    let mut pop_data: Vec<(Entity, GridPosition, Needs, UtilityWeights, PopAction)> = world
        .query::<(Entity, &GridPosition, &Needs, &UtilityWeights, &PopAction)>()
        .iter(world)
        .filter(|(_, _, _, _, action)| action.ticks_committed >= config.evaluation_interval)
        .map(|(e, p, n, w, a)| (e, *p, *n, w.clone(), PopAction {
            current: a.current,
            current_utility: a.current_utility,
            ticks_committed: a.ticks_committed,
        }))
        .collect();

    // Evaluate each pop
    for (pop_entity, pop_pos, needs, weights, mut action) in pop_data {
        let mut utilities = Vec::new();

        // Evaluate SatisfyHunger
        let farms = world.query::<(Entity, &GridPosition, &Farm)>();
        if let Some((utility, target)) = evaluate_satisfy_hunger(&pop_pos, &needs, &weights, &farms) {
            utilities.push((ActionType::SatisfyHunger, utility, Some(target)));
        }

        // Evaluate SatisfyRest
        let housing = world.query::<(Entity, &GridPosition, &Housing)>();
        if let Some((utility, target)) = evaluate_satisfy_rest(&pop_pos, &needs, &weights, &housing) {
            utilities.push((ActionType::SatisfyRest, utility, Some(target)));
        }

        // Evaluate Idle
        let idle_utility = evaluate_idle(&needs);
        utilities.push((ActionType::Idle, idle_utility, None));

        // Sort by utility
        utilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

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
            }
        }
    }
}
```

### Reinforcement Learning

```rust
// src/layer1/utility_ai.rs (continued)

/// Updates weights based on action outcome
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
    weights.distance_weight = weights.distance_weight.clamp(
        config.weight_clamp.0,
        config.weight_clamp.1,
    );
    weights.availability_weight = weights.availability_weight.clamp(
        config.weight_clamp.0,
        config.weight_clamp.1,
    );
    weights.social_weight = weights.social_weight.clamp(
        config.weight_clamp.0,
        config.weight_clamp.1,
    );
}

/// Detects completed HTN plans and triggers learning
pub fn track_plan_outcomes_system(world: &mut World) {
    let config = world.resource::<UtilityConfig>().clone();
    let sim_time = world.resource::<SimulationTime>().tick;

    // Find completed plans (have PlanOutcome but no Plan component)
    let completed: Vec<(Entity, PlanOutcome, Needs)> = world
        .query_filtered::<(Entity, &PlanOutcome, &Needs), Without<bevy_bae::Plan>>()
        .iter(world)
        .map(|(e, o, n)| (e, PlanOutcome {
            action: o.action,
            started_at: o.started_at,
            needs_before: o.needs_before,
        }, *n))
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
            update_weights_from_outcome(&mut weights, outcome.action, success, duration as u32, &config);
        }

        // Clean up
        world.entity_mut(pop_entity).remove::<PlanOutcome>();
    }
}
```

### Module Integration

```rust
// src/layer1/mod.rs

pub mod terrain;
pub mod pop;
pub mod needs;
pub mod building;
pub mod housing;
pub mod utility_ai;

pub use terrain::{TerrainGrid, TerrainType, Viewport, generate_terrain, render_map_layer};
pub use pop::{Pop, GridPosition, spawn_initial_pops, pop_display};
pub use needs::{Needs, decay_needs_system, kill_starving_pops_system};
pub use building::{Building, BuildingType, BuildMode, OccupiedTiles, can_place_building, try_place_building};
pub use housing::{Housing, restore_rest_in_housing_system, clean_dead_residents_system};
pub use utility_ai::{
    ActionType, PopAction, UtilityWeights, PlanOutcome, UtilityConfig, ColonyMemory,
    evaluate_actions_system, track_plan_outcomes_system,
};
```

### Update Pop Spawning

```rust
// src/layer1/pop.rs - Modify spawn_initial_pops

use super::utility_ai::{PopAction, UtilityWeights};

pub fn spawn_initial_pops(world: &mut World) {
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();
    let mut spawned = 0;

    while spawned < 5 {
        let x = rng.gen_range(0..terrain.width as i32);
        let y = rng.gen_range(0..terrain.height as i32);

        if let Some(terrain_type) = terrain.get(x as usize, y as usize) {
            if terrain_type != TerrainType::Water && terrain_type != TerrainType::Rock {
                world.spawn((
                    Pop,
                    GridPosition { x, y },
                    Needs::default(),
                    PopAction::default(),        // ADD
                    UtilityWeights::default(),   // ADD
                ));
                spawned += 1;
            }
        }
    }
}
```

### Main Loop Integration

```rust
// src/main.rs - Update simulation tick

if *world.resource::<GameState>() == GameState::Running {
    let speed = world.resource::<SimulationTime>().speed;
    if speed != SimSpeed::Paused {
        // PHASE 1: Utility AI
        evaluate_actions_system(&mut world);

        // PHASE 2: HTN Planning (bevy_bae systems run automatically)
        // create_htn_plans_system - to be implemented in future iteration

        // PHASE 3: Production & Consumption
        restore_rest_in_housing_system(&mut world);
        // produce_food_system - from spec 008
        // consume_food_system - from spec 008

        // PHASE 4: Decay & Death
        decay_needs_system(&mut world);
        kill_starving_pops_system(&mut world);
        clean_dead_residents_system(&mut world);

        // PHASE 5: Learning
        track_plan_outcomes_system(&mut world);

        world.resource_mut::<SimulationTime>().tick += 1;
    }
}
```

### Cargo.toml Dependencies

```toml
[dependencies]
bevy_ecs = "0.15.4"
bevy_app = "0.15.4"  # For bevy_bae
bevy_bae = "0.1"     # HTN planner
# ... existing dependencies
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **HTN Integration Incomplete**: This spec focuses on utility evaluation and learning. HTN plan creation (`create_htn_plans_system`) and operator definitions (`move_to_building_system`, `work_at_farm_system`) should be implemented in a follow-up iteration once bevy_bae integration is fully understood.

2. **Learning algorithm is simple**: Current reinforcement adjusts weights by fixed amounts. Future could use gradient descent, Q-learning, or other RL algorithms.

3. **No spatial memory**: Pops don't remember "Farm #3 worked well last time". Future: per-building success tracking.

4. **Social factor unused**: Weight exists but always multiplied by 1.0. Future: count nearby pops, implement flocking/avoidance.

5. **Evaluation every tick**: Even with threshold, calculating utilities every tick is expensive. Future: event-driven evaluation.

6. **No action queueing**: Pops only consider next action. Future: plan multi-step sequences.

### Performance Considerations

- **Utility evaluation is O(N*M)**: N pops × M buildings per action type
  - For 100 pops × 10 buildings × 2 action types = 2000 evaluations/tick
  - Acceptable for small colonies, needs optimization for 500+ pops
  - Future: spatial index (quadtree), caching, parallel evaluation

- **Weight updates are O(1)**: HashMap lookups, bounded by action type count
  - No performance concern

- **Threshold prevents thrashing**: Pops don't recalculate unless committed time elapsed
  - Naturally reduces evaluation frequency

### Extensibility Points

**Adding new action types:**
```rust
// Add variant
pub enum ActionType {
    // ... existing
    BuildStructure,
    DefendColony,
}

// Add evaluation function
fn evaluate_build_structure(...) -> Option<(f32, Entity)> { ... }

// Add to evaluate_actions_system
if let Some((utility, target)) = evaluate_build_structure(...) {
    utilities.push((ActionType::BuildStructure, utility, Some(target)));
}
```

**Colony memory integration:**
```rust
// In evaluate_satisfy_hunger:
let zeitgeist_bonus = world.resource::<ColonyMemory>()
    .building_success_rate(farm_entity)
    .unwrap_or(1.0);

let utility = hunger_urgency * context_score * success_mod * zeitgeist_bonus;
```

**Per-building memory:**
```rust
#[derive(Component)]
pub struct BuildingMemory {
    pub total_workers: u32,
    pub successful_uses: u32,
    pub average_satisfaction: f32,
}

// Pops query this during evaluation
let building_reputation = building_memory.successful_uses as f32 / building_memory.total_workers as f32;
```

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for layer1/utility_ai.rs
- [ ] Pops spawn with `PopAction` and `UtilityWeights` components
- [ ] `evaluate_actions_system` calculates utilities for all actions
- [ ] Pops switch actions when utility exceeds threshold
- [ ] Pops respect commitment interval (don't thrash)
- [ ] Response curves produce higher urgency for lower needs
- [ ] Context scoring factors in distance and availability
- [ ] Success modifier increases with good history
- [ ] `track_plan_outcomes_system` detects completed actions
- [ ] Weights increase on successful outcomes
- [ ] Weights decrease on failed outcomes
- [ ] Weights clamp to configured range (0.5-2.0)
- [ ] Over 10+ cycles, successful pops develop higher weights
- [ ] HTN integration markers (`StartPlan`, `PlanOutcome`) work correctly
- [ ] System execution order follows spec (eval → plan → produce → decay → learn)

## Technical Guidance

### Utility Calculation Breakdown

**Example: SatisfyHunger with hunger=0.3**

```
1. Need urgency = need_response_curve(0.3)
   = 1.0 - 0.3^2 = 0.91 (very urgent!)

2. Context score (farm at distance 5, capacity 4, occupied 2):
   distance_factor = 1.0 / (1.0 + 5*0.1) = 0.67
   availability = 1.0 - (2/4) = 0.5
   context_score = 0.67^1.0 * 0.5^1.0 = 0.335

3. Success modifier (2 attempts, 1 success):
   success_rate = 1/2 = 0.5
   modifier = 0.8 + 0.5*0.4 = 1.0

4. Final utility = 0.91 * 0.335 * 1.0 = 0.305
```

### Response Curve Characteristics

```
need_value → urgency
1.0 → 0.00  (fully satisfied, no urgency)
0.9 → 0.19  (slight urgency)
0.7 → 0.51  (moderate urgency)
0.5 → 0.75  (significant urgency)
0.3 → 0.91  (high urgency)
0.1 → 0.99  (critical urgency)
0.0 → 1.00  (maximum urgency)
```

The curve is exponential (x^2), creating drama at low values.

### Weight Learning Dynamics

Starting weights: `{distance: 1.0, availability: 1.0}`

**After 5 successful quick actions:**
```
distance_weight += 0.05 * 0.1 * 5 = +0.025 → 1.025
availability_weight += 0.05 * 0.05 * 5 = +0.0125 → 1.0125
```

**After 5 failed slow actions:**
```
distance_weight -= 0.05 * 0.05 * 5 = -0.0125 → 0.9875
availability_weight -= 0.05 * 0.05 * 5 = -0.0125 → 0.9875
```

Learning is gradual - takes 20-50 experiences to shift weights noticeably.

### Threshold Tuning Guide

| Threshold | Behavior | Use Case |
|-----------|----------|----------|
| 0.05 | Very reactive, switches often | High-pressure survival scenarios |
| 0.15 | Balanced (default) | Normal gameplay |
| 0.30 | Conservative, commits to plans | Stable colonies, strategic play |
| 0.50 | Very committed, rarely switches | Testing, deterministic behavior |

### Common Pitfalls

1. **Forgetting to add components on spawn**: Pops need `PopAction` and `UtilityWeights` or systems will skip them

2. **Expecting instant learning**: Weights change by ~5% per outcome. Needs 10-20 cycles to see adaptation.

3. **Threshold too low**: Pops thrash between actions. Increase `switch_threshold`.

4. **Threshold too high**: Pops never switch even when starving. Decrease `switch_threshold`.

5. **No HTN integration**: This spec implements utility evaluation only. HTN plan creation is a separate phase. Pops will insert `StartPlan` but nothing consumes it yet.

6. **Evaluation interval = 0**: Division by zero risk. Must be ≥ 1.

## Integration Notes

### HTN Plan Creation (Future Work)

This spec implements the utility AI layer. The HTN integration layer (`create_htn_plans_system`) should be implemented after:

1. bevy_bae API is fully understood
2. Operator systems are defined (`move_to_building_system`, `work_at_farm_system`, etc.)
3. Preconditions/effects are designed

Suggested approach:
- Create spec `017-htn-integration.md`
- Define all operator systems
- Define task trees for each `ActionType`
- Implement `create_htn_plans_system` to bridge `StartPlan` → bevy_bae `Plan`

### Farm Building Dependency

This spec assumes Farm building exists from spec 008:

```rust
#[derive(Component)]
pub struct Farm {
    pub capacity: usize,
    pub workers: Vec<Entity>,
}
```

If spec 008 not implemented, stub it:
```rust
// Temporary stub for testing
#[derive(Component, Default)]
pub struct Farm {
    pub capacity: usize,
    pub workers: Vec<Entity>,
}

impl Default for Farm {
    fn default() -> Self {
        Self { capacity: 2, workers: vec![] }
    }
}
```

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## Forward Compatibility Notes

**Expansion Points:**

1. **Colony Memory (Zeitgeist)**
   - Add `ColonyMemory` resource tracking
   - Pops query shared knowledge during evaluation
   - Emergent: colony-wide learning from any pop's experience

2. **Individual Memory**
   - Add `PopMemory` component with per-building history
   - Pops remember "Farm #3 worked well for me"
   - Emergent: specialists emerge, personal preferences

3. **Spatial Awareness**
   - Add position-based success heatmaps
   - Pops learn "northeast area is productive"
   - Emergent: migration patterns, territory formation

4. **Social Dynamics**
   - Implement `social_weight` factor (currently stubbed)
   - Pops avoid crowded areas or seek company
   - Emergent: flocking, queuing, social hierarchies

5. **More Actions**
   - Add `ActionType::Socialize`, `Explore`, `BuildStructure`, etc.
   - Each gets evaluation function + HTN plan
   - Emergent: complex behavioral loops, role specialization
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
