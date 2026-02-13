# 108: Workplace Hazards

## Overview

Life on the frontier is dangerous. While `ActionType` defines `danger_level` for various tasks (Work, Repair, Tame, Slaughter), the actual execution of these hazards is inconsistent. Currently, only `Work` and `Repair` trigger accidents via `work_execution_system`. Dangerous tasks like `Tame` (animals bite) and `Slaughter` (sharp tools) do not inflict injury, making them safer than intended.

This spec standardizes hazard execution by introducing a shared `hazards` module and applying it to all dangerous actions.

## Dependencies

- `009` — Job System (Completed)
- `034` — Pop Health (Completed)
- `075` — Animal Husbandry (Completed)

## RED Phase: Tests First

These tests verify that dangerous actions cause damage over time.

```rust
// src/layer1/hazards_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::utility_ai::ActionType;
    use crate::layer1::health::Health;
    use crate::layer1::husbandry::{tame_execution_system, Tame};
    use crate::layer1::execution::{MovementTarget, AtTarget};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::fauna::{Fauna, FaunaType};

    #[test]
    fn test_taming_is_dangerous() {
        let mut world = World::new();
        // Setup systems/resources
        // ...

        // Create a Pop attempting to tame a Wolf (high danger)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Health { current: 100.0, max: 100.0 },
            MovementTarget {
                target_entity: Entity::PLACEHOLDER, // Will need a real entity
                target_position: GridPosition { x: 0, y: 1 },
                for_action: ActionType::Tame,
            },
            AtTarget,
        )).id();

        // Create the animal
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Fix target entity reference
        world.get_mut::<MovementTarget>(pop).unwrap().target_entity = animal;

        // Force hazard RNG to always trigger (requires mocking or looping)
        // For TDD, we loop enough times to statistically guarantee a hit
        // Tame danger = 0.5% per tick. 1000 ticks = ~99% chance of at least one accident.

        let mut injured = false;
        for _ in 0..1000 {
            // Reset health to survive multiple hits
            if let Some(mut h) = world.get_mut::<Health>(pop) {
                if h.current < 100.0 {
                    injured = true;
                    h.current = 100.0; // Heal
                }
            }

            tame_execution_system(&mut world);
        }

        assert!(injured, "Pop should have taken damage from taming accidents");
    }

    #[test]
    fn test_slaughter_is_dangerous() {
        // Similar to Tame, but for Slaughter action
        // Requires implementing slaughter_execution_system first if missing
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `src/layer1/hazards.rs`

Move the hazard logic from `src/layer1/execution.rs` to a shared module.

```rust
// src/layer1/hazards.rs

use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::utility_ai::ActionType;
use crate::layer1::health::Health;
use crate::shared::log::MessageLog;

pub fn handle_workplace_hazards(world: &mut World, pop_entity: Entity, action_type: ActionType) {
    let danger = action_type.danger_level();
    let mut rng = rand::thread_rng();

    if rng.gen_bool(danger) {
        let damage = action_type.accident_damage();

        // Apply damage if pop has Health
        if let Some(mut health) = world.get_mut::<Health>(pop_entity) {
            health.take_damage(damage);

            // Log accident
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add(format!("ACCIDENT: Worker injured while {:?}! (-{} HP)", action_type, damage));
            }
        }
    }
}
```

### 2. Update `src/layer1/execution.rs`

Remove the local `handle_workplace_hazards` and use the shared one.

```rust
use crate::layer1::hazards::handle_workplace_hazards;

fn handle_post_work_effects(...) {
    // ...
    handle_workplace_hazards(world, pop_entity, action_type);
    // ...
}
```

### 3. Update `src/layer1/husbandry.rs`

Call `handle_workplace_hazards` in `tame_execution_system`.

```rust
use crate::layer1::hazards::handle_workplace_hazards;

pub fn tame_execution_system(world: &mut World) {
    // ... setup ...

    for (pop_entity, designation_entity) in tamers {
        // ... taming logic ...

        // Apply Hazards
        handle_workplace_hazards(world, pop_entity, ActionType::Tame);

        // ... cleanup ...
    }
}
```

## REFACTOR Phase: Quality & Design

- **Hazard Modifiers**: In the future, `Skills` (e.g., Husbandry) or `Equipment` (Armor) could reduce `danger_level` or `accident_damage`.
- **Severity Variance**: Currently damage is fixed. It could vary (Critical Fail vs Minor Scrape).
- **Juice**: Trigger screen shake or particles (blood?) on accident.

## Acceptance Criteria

- [ ] `handle_workplace_hazards` is refactored into a public shared module.
- [ ] `tame_execution_system` applies hazard damage.
- [ ] `work_execution_system` continues to apply hazard damage.
- [ ] Tests for Taming hazards pass.

## Technical Guidance

- Ensure `hazards.rs` is registered in `layer1/mod.rs`.
- Be careful with `world` borrowing in `tame_execution_system`. `handle_workplace_hazards` takes `&mut World`, so you cannot hold references to components while calling it. You must collect entities first (which `tame_execution_system` already does).

## Questions

*Builder: add questions here if spec is unclear.*
