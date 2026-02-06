# 035: Workplace Hazards

## Overview

Working in the frontier is dangerous. Mining rocks and felling trees carries a risk of injury. This spec introduces a `danger_level` to `ActionType` and integrates with the `Health` system to inflict damage when accidents occur.

This adds tension to the game: high productivity (lots of workers) now comes with a medical cost.

## Dependencies

- `034` — Pop Health (Backlog)
- `009` — Job System (Completed, `execution.rs`)
- `033` — Fire Propagation (Backlog) - *Optional interaction*

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/hazards_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::utility_ai::ActionType;
    use crate::layer1::health::Health;
    use crate::layer1::execution::work_execution_system;
    // ... other imports

    #[test]
    fn test_action_danger_levels() {
        // Mining and Forestry should be dangerous
        assert!(ActionType::Work.danger_level() > 0.0);

        // Sleeping and Eating should be safe
        assert_eq!(ActionType::SatisfyHunger.danger_level(), 0.0);
        assert_eq!(ActionType::SatisfyRest.danger_level(), 0.0);
    }

    #[test]
    fn test_work_accident_damage() {
        // This test is probabilistic, so we might need to mock RNG or
        // set the danger/accident chance very high for the test.
        // Alternatively, expose a helper to test the logic deterministically.

        let mut world = World::new();
        // Setup pop with full health working on a dangerous task
        // ...

        // Force an accident (requires refactoring execution to allow mocking or dependency injection)
        // OR: Loop many times to statistically verify damage occurs.

        let initial_health = 100.0;
        let mut took_damage = false;

        for _ in 0..1000 {
            // Run work system
            // Check health
            // If health < initial, took_damage = true; break;
            // Reset health
        }

        assert!(took_damage, "Dangerous work should eventually cause damage");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ActionType`

Modify `src/layer1/utility_ai.rs`:

```rust
impl ActionType {
    pub fn danger_level(&self) -> f64 {
        match self {
            Self::Work => 0.001, // 0.1% chance per tick
            _ => 0.0,
        }
    }

    pub fn accident_damage(&self) -> f32 {
        match self {
            Self::Work => 10.0, // 10 HP damage
            _ => 0.0,
        }
    }
}
```

### 2. Update `work_execution_system`

Modify `src/layer1/execution.rs` to include hazard checks.

```rust
use crate::layer1::health::Health;

pub fn work_execution_system(world: &mut World) {
    // ... existing setup ...

    // We need to query for Health now too
    // Note: This requires changing the query signature which might break existing code if not careful
    // or we can do a separate pass for hazards.

    // Better: Integrate into the main loop
    let workers: Vec<(Entity, Entity, Option<&mut Health>)> = ...;

    for (pop_entity, designation_entity, health_opt) in workers {
        // ... work logic ...

        // Hazard Check
        if let Some(mut health) = health_opt {
             let danger = ActionType::Work.danger_level();
             let mut rng = rand::thread_rng();
             if rng.gen_bool(danger) {
                 let damage = ActionType::Work.accident_damage();
                 health.take_damage(damage);

                 // Log it
                 if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                     log.add(format!("ACCIDENT: Worker injured! (-{} HP)", damage));
                 }
             }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Designation Specifics**: `ActionType::Work` is generic. We might want Mining to be more dangerous than Forestry.
    - *Solution*: Pass `DesignationType` to `danger_level`.
- **Mocking RNG**: Refactor `work_execution_system` to accept an `Rng` trait for deterministic testing.
- **Safety Gear**: Future integration with `Tool` system (e.g. Helmets reduce damage).

## Acceptance Criteria

- [ ] `ActionType::danger_level()` implemented.
- [ ] `ActionType::accident_damage()` implemented.
- [ ] Working pops take damage occasionally.
- [ ] Accidents are logged to `MessageLog`.
- [ ] Tests pass.
- [ ] Safe actions (Eating, Sleeping) never cause accidents.

## Technical Guidance

- Be careful with `world.query` vs `world.get_resource`.
- `work_execution_system` is currently getting complex. Consider splitting "Progress Logic" and "Hazard Logic" if it gets too messy, though keeping them together saves a query iteration.
- Ensure `Health` component is optional in the query if we want to support non-health entities working (e.g. Robots), though currently only Pops work.

## Questions

*Builder: add questions here if spec is unclear.*
