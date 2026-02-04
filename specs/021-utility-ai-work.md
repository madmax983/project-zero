# 021: Utility AI Work Action

## Overview

Integrate resource gathering into the Utility AI system (Spec 016). Pops will autonomously find and work on active designations (Mining, Forestry) when their survival needs are satisfied.

## Dependencies

- `016` — Utility AI System (must be implemented first)
- `017` — Designation System (for `Designation` component)
- `018` — Mining (for `MiningProgress`)
- `019` — Forestry (for `ForestryProgress`)

## RED Phase: Tests First

Write these tests in `src/layer1/utility_ai_work_tests.rs` (or extend `utility_ai.rs` tests).

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::utility_ai::{ActionType, evaluate_work, UtilityWeights, calculate_context_score};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::pop::{GridPosition, Pop};
    use crate::layer1::needs::Needs;

    #[test]
    fn test_action_type_work_variant() {
        let work = ActionType::Work;
        assert_eq!(work, ActionType::Work);
    }

    #[test]
    fn test_evaluate_work_finds_designation() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn a designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 0 },
        )).id();

        let designations = world.query::<(Entity, &GridPosition, &Designation)>();

        let result = evaluate_work(&pop_pos, &weights, &designations);

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, designation);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_work_prioritizes_closest() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights {
            distance_weight: 2.0, // High preference for close work
            ..Default::default()
        };

        // Far designation
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 20, y: 0 },
        ));

        // Close designation
        let close = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 2, y: 0 },
        )).id();

        let designations = world.query::<(Entity, &GridPosition, &Designation)>();

        let (_, target) = evaluate_work(&pop_pos, &weights, &designations).unwrap();
        assert_eq!(target, close);
    }

    #[test]
    fn test_evaluate_work_no_designations() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let designations = world.query::<(Entity, &GridPosition, &Designation)>();

        let result = evaluate_work(&pop_pos, &weights, &designations);
        assert!(result.is_none());
    }

    // Integration test ensuring Work is chosen over Idle when needs are met
    #[test]
    fn test_work_beats_idle() {
        // ... setup world with satisfied pop and a designation ...
        // evaluate_actions_system(&mut world);
        // assert_eq!(pop_action.current, ActionType::Work);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ActionType
```rust
// src/layer1/utility_ai.rs
pub enum ActionType {
    // ...
    Work,
}
```

### 2. Implement Evaluation
```rust
// src/layer1/utility_ai.rs

use crate::layer1::designation::Designation;

pub fn evaluate_work(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: &Query<(Entity, &GridPosition, &Designation)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    // Base utility for working (could depend on traits later)
    let base_utility = 0.5;

    for (entity, pos, _) in designations.iter() {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity 1 (one worker per tile usually)
            0, // Occupied 0 (simplified for now, or check if someone else targeted it)
            weights
        );

        let success = calculate_success_modifier(ActionType::Work, weights);
        let utility = base_utility * context * success;

        if best.is_none() || utility > best.unwrap().0 {
            best = Some((utility, entity));
        }
    }
    best
}
```

### 3. Integrate into System
Update `evaluate_actions_system` to include `evaluate_work`.

```rust
// In evaluate_actions_system loop:
let designations = world.query::<(Entity, &GridPosition, &Designation)>();
if let Some((utility, target)) = evaluate_work(&pop_pos, &weights, &designations) {
    utilities.push((ActionType::Work, utility, Some(target)));
}
```

## REFACTOR Phase: Quality & Design

- **Task Locking**: Currently `evaluate_work` ignores if another pop is already targeting the designation. We need a way to reserve designations (maybe `TargetedBy` component or check `PopAction` of others).
- **Job Priorities**: Different designation types (Mine vs Build vs Chop) should have different priorities or utilities based on colony needs (e.g., "We need Food" -> farm work utility boost).
- **Skill Factor**: Pops with high Mining skill should prefer Mining designations.

## Acceptance Criteria

- [ ] `ActionType::Work` exists.
- [ ] Pops choose `Work` when needs are satisfied and designations exist.
- [ ] Pops prefer closer designations.
- [ ] `Work` action properly targets the designation entity.
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **Reservation Issue**: In the Minimal Phase, multiple pops might target the same tile. This is acceptable for now (they will just crowd). The `HTN` layer (next spec) or a "Claim" system will handle the actual exclusion.
- **Performance**: Querying all designations for all pops is O(N*M). If designations are many (1000+), this needs spatial partitioning. For MVP, it's fine.
