# 1261: The Sleepless Caste

## 1. Overview

A segment of your society gives up their right to rest to become hyper-productive, fundamentally altering the colony's culture and rhythm. A new genetic trait or cybernetic implant called "Insomnia Drive" eliminates the Rest need entirely, increasing productivity by 30%. However, these pops generate double the amount of Stress and require massive amounts of advanced Leisure facilities to prevent violent breakdowns.

## 2. Dependencies

- Layer 1 Needs System (`src/layer1/needs.rs`)
- Layer 1 Traits System (`src/layer1/psychology/traits.rs`)
- Layer 1 Execution System (`src/layer1/execution/general_work.rs`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use crate::layer1::needs::Needs;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use crate::layer1::social::unrest::MentalState;
    use crate::layer1::Pop;
    use crate::layer1::psychology::needs::decay_needs_system;
    use crate::layer1::execution::general_work::work_execution_system;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_insomnia_drive_eliminates_rest_decay() {
        let mut world = World::new();

        let mut traits = Traits::default();
        traits.add(Trait::InsomniaDrive);

        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 1.0,
            },
            traits,
        )).id();

        // Run metabolism system
        world.run_system_once(decay_needs_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        // Hunger should decay normally
        assert!(needs.hunger < 1.0);
        // Rest should not decay at all due to Insomnia Drive
        assert_eq!(needs.rest, 1.0);
    }

    #[test]
    fn test_insomnia_drive_increases_productivity() {
        let mut world = World::new();
        // Setup a pop with normal traits and one with InsomniaDrive
        // Verify that work_execution_system or evaluate_actions_system
        // applies a 1.3x multiplier to work progress for the Sleepless pop.
    }

    #[test]
    fn test_insomnia_drive_doubles_stress_generation() {
        // Setup stress/leisure metabolism
        // Verify that the leisure need drops twice as fast or stress accumulates twice as fast
        // for pops with the InsomniaDrive trait.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/psychology/traits.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trait {
    // ... existing traits
    InsomniaDrive,
}

// In src/layer1/psychology/needs.rs (decay_needs_system)
pub fn decay_needs_system(mut query: Query<(&mut Needs, Option<&Traits>)>) {
    let mut query = world.query::<(&mut Needs, Option<&Traits>)>();
    for (mut needs, traits) in query.iter_mut(world) {
        needs.hunger -= 0.01;

        // Insomnia Drive eliminates rest decay and doubles leisure decay (stress)
        if let Some(t) = traits {
            if t.has(Trait::InsomniaDrive) {
                needs.leisure -= 0.02; // Double stress
            } else {
                needs.rest -= 0.01;
                needs.leisure -= 0.01;
            }
        } else {
            needs.rest -= 0.01;
            needs.leisure -= 0.01;
        }

        needs.hygiene -= 0.01;
    }
}

// In src/layer1/execution/general_work.rs (or where work progress is calculated)
pub fn calculate_work_speed(traits: Option<&Traits>) -> f32 {
    let mut speed = 1.0;
    if let Some(t) = traits {
        if t.has(Trait::InsomniaDrive) {
            speed *= 1.3;
        }
    }
    speed
}
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the 1.3x productivity multiplier stacks correctly with other buffs or debuffs (e.g., from tools or low morale).
- The "double stress" mechanic should be integrated smoothly into the existing `Morale` or `Needs` calculation so it doesn't break balance if other traits also modify stress.
- Provide a UI indication for Sleepless pops so the player knows why they are suddenly rioting.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `InsomniaDrive` trait prevents `rest` decay and doubles `leisure` decay.
- [ ] `InsomniaDrive` increases work progress by 30%.

## 7. Technical Guidance

- Modify `src/layer1/psychology/traits.rs` to add `InsomniaDrive`.
- Update `decay_needs_system` in `src/layer1/psychology/needs.rs` to handle the need decay logic.
- Update `work_execution_system` or the relevant helper function in `src/layer1/execution/general_work.rs` to apply the 1.3x work speed multiplier.
- Be careful with `Option<&Traits>` in queries to ensure pops without traits default to normal behavior.

## 8. Questions
*Builder: add questions here if spec is unclear.*

*Architect: Addressed Builder's question regarding file paths and function names on 2026-06-03.*
