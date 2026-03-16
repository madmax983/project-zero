# Synthetic Apathy (Spec 472)

## Overview
Basic "Synth" pops have 100% work efficiency and no morale needs. However, they possess "Apathy", meaning they will rigidly execute their assigned tasks but completely ignore adjacent emergencies like fires or injured colonists unless explicitly ordered.

## Dependencies
- `016` Utility AI System (Implemented)
- `411` Machine Awakening (Implemented)
- `033` Fire Propagation (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_synth_pop_ignores_fire_emergency() {
        let mut world = World::new();

        // Arrange
        let synth = world.spawn((
            PopBundle::default(),
            Trait::Synth,
            GridPosition { x: 10, y: 10 },
            UtilityWeights::default(),
        )).id();

        let fire = world.spawn((
            Fire,
            GridPosition { x: 10, y: 11 }, // Adjacent
        )).id();

        let mut ai_buffer = UtilityAIBuffer::default();
        ai_buffer.candidates.push(ScorableCandidate {
            entity: fire,
            action_type: ActionType::ExtinguishFire,
        });

        // Add a hauling job further away
        let hauling_job = world.spawn((
            JobItem,
            GridPosition { x: 20, y: 20 },
        )).id();
        ai_buffer.candidates.push(ScorableCandidate {
            entity: hauling_job,
            action_type: ActionType::Haul,
        });

        world.insert_resource(ai_buffer);

        // Act
        world.run_system_once(evaluate_actions_system).unwrap();

        // Assert
        let action = world.get::<CurrentAction>(synth).unwrap();
        // The synth should choose to haul rather than extinguish the adjacent fire
        assert_eq!(action.action_type, ActionType::Haul);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

// In src/layer1/utility_ai.rs or a new layer1/tech/synth.rs

pub fn evaluate_actions_system(
    mut ai_buffer: ResMut<UtilityAIBuffer>,
    mut pops: Query<(Entity, &mut CurrentAction, Option<&Trait>)>,
) {
    for (pop_entity, mut current_action, trait_opt) in pops.iter_mut() {
        let is_synth = trait_opt.map_or(false, |t| *t == Trait::Synth);
        let mut best_score = 0.0;
        let mut best_action = ActionType::Idle;

        for candidate in &ai_buffer.candidates {
            // Synths heavily penalize emergency actions unless explicitly ordered
            let mut score = calculate_base_score(candidate.action_type);

            if is_synth && is_emergency(candidate.action_type) {
                score *= 0.0; // Apathy: 0 priority for emergencies
            }

            if score > best_score {
                best_score = score;
                best_action = candidate.action_type.clone();
            }
        }

        current_action.action_type = best_action;
    }
}

fn is_emergency(action: &ActionType) -> bool {
    matches!(action, ActionType::ExtinguishFire | ActionType::TreatWounds | ActionType::Flee)
}

fn calculate_base_score(action: ActionType) -> f32 {
    // simplified base scoring for illustration
    match action {
        ActionType::ExtinguishFire => 0.9,
        ActionType::Haul => 0.5,
        _ => 0.1,
    }
}
```

## REFACTOR Phase: Quality & Design
- **Action Type System**: The `is_emergency` check should ideally be a property of the `ActionType` definition itself rather than a hardcoded match statement.
- **Explicit Override**: Add a "Direct Command" feature (potentially tying into `The Direct Link` spec) where the player can explicitly override a Synth's apathy by forcefully assigning the task, bypassing the Utility AI penalty.
- **Awakening Interaction**: A Synth that undergoes "Machine Awakening" (Spec 411) should lose the Apathy modifier and begin evaluating emergencies normally.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Synths evaluate emergency actions with a 0 modifier in the Utility AI.
- [ ] Synths will continue working on routine tasks (like hauling) even when adjacent to life-threatening emergencies.

## Technical Guidance
- Modify the existing `UtilityAIBuffer` evaluation loop. Do not rewrite the whole AI system.
- `Trait::Synth` should also be hooked up to disable or ignore `Needs::morale` decay in the `metabolism_system`.

## Questions
*Builder: add questions here if spec is unclear.*
