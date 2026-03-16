# Specification 466: Synthetic Apathy

## 1. Overview
Building the perfect, emotionless workforce has a dark side. Basic "Synth" pops have 100% efficiency and no morale needs, but possess a permanent "Apathy" trait. They will rigidly execute their assigned tasks but completely ignore adjacent emergencies (fires, injuries, raids) unless explicitly ordered. They will step over a dying colonist to deliver a piece of coal.

This adds tension: Flawless, cheap baseline efficiency vs. the total lack of emergent problem-solving and reactive self-preservation.

## 2. Dependencies
- `016` Utility AI System (`evaluate_actions_system`)
- `034` Pop Health and Damage
- `031` Pop Morale (or lack thereof)
- `033` Fire Propagation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Needs, Traits};
    use crate::layer1::utility_ai::{UtilityAIBuffer, evaluate_single_pop};
    use crate::layer1::actions::ActionType;

    fn setup_world() -> World {
        let mut world = World::new();
        world
    }

    #[test]
    fn test_synth_has_no_morale_needs() {
        let mut world = setup_world();

        let synth = world.spawn((
            Pop,
            Needs::default(),
            Traits::from(vec!["Synth"]),
        )).id();

        // Morale decay system shouldn't touch synths, or their target morale is always 1.0
        // (Implementation can vary, but concept is tested here)
    }

    #[test]
    fn test_apathy_ignores_emergencies() {
        let mut world = setup_world();

        let synth = world.spawn((
            Pop,
            Traits::from(vec!["Synth"]), // Synths have apathy implicitly or explicitly
            // ... required components for AI
        )).id();

        let mut buffer = UtilityAIBuffer::new();
        // Add an emergency action (e.g. FleeFire) and a normal action (e.g. Work)
        buffer.add_candidate(synth, ActionType::Flee, 100.0); // Normally high priority
        buffer.add_candidate(synth, ActionType::Work, 50.0);

        evaluate_single_pop(&world, synth, &mut buffer);

        let best_action = buffer.get_best_action();

        // Synth should ignore the emergency Flee action and choose Work
        assert_eq!(best_action, Some(ActionType::Work));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/pop/traits.rs
// (Assuming a Traits component exists, or adding it)
// If adding a specific component:
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Synth;

#[derive(Component)]
pub struct Apathy;

// src/layer1/utility_ai.rs
// Modifying the evaluation logic
use crate::layer1::actions::ActionType;

pub fn evaluate_single_pop(world: &World, entity: Entity, buffer: &mut UtilityAIBuffer) {
    let has_apathy = world.get::<Apathy>(entity).is_some() || world.get::<Synth>(entity).is_some();

    // Iterate through candidates and modify scores
    for candidate in buffer.candidates_mut() {
        if has_apathy {
            match candidate.action {
                ActionType::Flee | ActionType::ExtinguishFire | ActionType::Rescue => {
                    // Reduce score to 0 or extremely low so it's ignored
                    candidate.score = 0.0;
                },
                _ => {} // Other tasks remain scored normally
            }
        }
    }

    // Proceed with normal best-action selection
    buffer.sort_and_select();
}
```

## 5. REFACTOR Phase: Quality & Design
- **Override Command**: Players should be able to explicitly command Synths to handle emergencies via the designation system. Apathy only ignores *autonomous* emergency reactions.
- **Trait Definition**: Consolidate `Synth` and `Apathy` if they always exist together, or keep them separate so organic Pops can also gain `Apathy` through trauma.
- **Visuals**: Synths walking through fire should probably take damage if they don't flee. Ensure the damage system doesn't rely on the Flee action to apply damage.

## 6. Acceptance Criteria (Testable!)
- [ ] `Synth` and `Apathy` components or traits exist.
- [ ] Pops with `Apathy` assign a 0.0 utility score to emergency actions (Flee, Rescue, ExtinguishFire).
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the modified `evaluate_single_pop` logic.

## 7. Technical Guidance
- Integration with the `UtilityAIBuffer` requires hooking into the scoring modification phase before final selection. Ensure you don't skip the step entirely.
- Ensure that an explicit player order (e.g., `ActionType::PlayerDesignated(ExtinguishFire)`) bypasses the Apathy filter.

## 8. Questions
*Builder: add questions here if spec is unclear.*
