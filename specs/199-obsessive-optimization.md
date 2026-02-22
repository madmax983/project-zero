# 199: Obsessive Optimization

## Overview

*"If it ain't broke, fix it until it is."*

This feature introduces a behavior where highly skilled engineers feel compelled to improve the efficiency of machines around them. While this can lead to significant productivity gains ("Optimized" status), it carries the risk of damaging the equipment ("Broken" status) if the optimization goes wrong.

This adds a risk/reward layer to maintaining a highly skilled workforce and creates emergent storytelling moments where your best engineer accidentally sabotages the power plant.

## Dependencies

- `004` — Building System (Target entities)
- `051` — Pop Skills (Skill level checks)
- `066` — Building Work AI (Action injection)
- `016` — Utility AI (Action evaluation)

## RED Phase: Tests First

Write these tests in `src/layer1/optimization_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::optimization::{Optimized, OptimizationResult, perform_optimization};
    use crate::layer1::structure::Structure;

    #[test]
    fn test_optimization_success_applies_component() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        // Perform optimization with FORCE_SUCCESS flag or high skill + lucky seed
        // For unit test, we might direct call the success logic
        let result = OptimizationResult::Success;
        perform_optimization(&mut world, building, result);

        let optimized = world.get::<Optimized>(building);
        assert!(optimized.is_some());
        assert_eq!(optimized.unwrap().efficiency_bonus, 0.10);
    }

    #[test]
    fn test_optimization_failure_damages_building() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::Failure;
        perform_optimization(&mut world, building, result);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0);
        assert!(world.get::<Optimized>(building).is_none());
    }

    #[test]
    fn test_critical_failure_breaks_building() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::CriticalFailure;
        perform_optimization(&mut world, building, result);

        let structure = world.get::<Structure>(building).unwrap();
        // Should be broken (0 HP or specific Broken component)
        // Assuming Structure logic handles <= 0 as broken
        assert!(structure.current_hp <= 0.0);
    }

    #[test]
    fn test_already_optimized_cannot_be_optimized_again() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Optimized { efficiency_bonus: 0.10 },
        )).id();

        // Logic should reject this target before action,
        // but perform_optimization should probably handle it gracefully or be idempotent
        perform_optimization(&mut world, building, OptimizationResult::Success);

        let optimized = world.get::<Optimized>(building).unwrap();
        assert_eq!(optimized.efficiency_bonus, 0.10); // Should not stack to 0.20
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/optimization.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::structure::Structure;

#[derive(Component, Debug, Clone, Default)]
pub struct Optimized {
    pub efficiency_bonus: f32, // e.g., 0.10 for +10%
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationResult {
    Success,
    CriticalSuccess,
    Failure,
    CriticalFailure,
}

pub fn perform_optimization(world: &mut World, target: Entity, result: OptimizationResult) {
    match result {
        OptimizationResult::Success => {
            if world.get::<Optimized>(target).is_none() {
                world.entity_mut(target).insert(Optimized { efficiency_bonus: 0.10 });
            }
        }
        OptimizationResult::CriticalSuccess => {
            if world.get::<Optimized>(target).is_none() {
                world.entity_mut(target).insert(Optimized { efficiency_bonus: 0.25 });
            }
        }
        OptimizationResult::Failure => {
            if let Some(mut structure) = world.get_mut::<Structure>(target) {
                structure.current_hp -= structure.max_hp * 0.25;
                if structure.current_hp < 0.0 { structure.current_hp = 0.0; }
            }
        }
        OptimizationResult::CriticalFailure => {
            if let Some(mut structure) = world.get_mut::<Structure>(target) {
                structure.current_hp = 0.0; // Breaks instantly
            }
        }
    }
}
```

### 2. Utility AI Action (`src/layer1/utility_ai.rs`)

Add `Tinker` to `ActionType`.

### 3. Evaluation Logic

In `evaluate_actions_system` (or a new `evaluate_optimization_system`):
- Filter Pops with `SkillType::Construction` or `SkillType::Crafting` level >= 3.
- Find nearby `Building` entities without `Optimized` component.
- Calculate Score:
  - Base Score: 0.3 (Low priority)
  - Modifiers: +0.1 per Level above 3. +0.2 if `Traits::Obsessive`.
- Return `ActionType::Tinker(target)`.

### 4. Execution Logic (`src/layer1/execution.rs`)

In `work_execution_system`:
- If `action == Tinker`:
  - Duration: 50 ticks.
  - On complete:
    - Calculate Result based on RNG and Skill Level.
    - Call `perform_optimization`.

## REFACTOR Phase: Quality & Design

- **Visual Feedback**: Add a "Sparks" particle effect during tinkering.
- **Notifications**: "Engineer X optimized the Generator!" or "Engineer Y broke the Generator!".
- **Trait Integration**:
  - `Obsessive`: Higher urge, higher success chance.
  - `Clumsy`: Higher failure chance.
  - `Lucky`: Higher critical success chance.
- **Skill XP**: Successful optimization should grant XP.

## Acceptance Criteria

- [ ] `Optimized` component exists.
- [ ] `perform_optimization` applies bonus on success and damage on failure.
- [ ] Engineers with high skill can select `Tinker` action.
- [ ] Buildings can be broken by critical failure.
- [ ] Optimized buildings produce more (integration with production systems).

## Technical Guidance

- Ensure `Structure` component handles damage correctly (does 0 HP trigger destruction or just "Broken" state?). If destruction, ensure we don't segfault accessing it later.
- For "Optimization increases production", modify `src/layer1/production.rs` (or wherever efficiency is calculated) to check for `Optimized` component and apply `1.0 + efficiency_bonus`.
