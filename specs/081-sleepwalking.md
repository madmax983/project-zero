# 081: Sleepwalking

## Overview

Stressed pops may enter a **Sleepwalking** state while sleeping. This state interrupts their rest and causes them to perform random, useless actions. They wander to a random target, perform a brief interaction (or nothing), and wake up confused, often without having recovered enough Rest.

This adds emergent behavior to the stress system and creates unique stories ("Why is the doctor in the stockpile at 3 AM?").

## Dependencies

- `005` — Pop Needs (Rest mechanism)
- `031` — Pop Morale (Stress tracking)
- `050` — Civil Unrest (Mental Break framework)
- `065` — Day/Night Cycle (Context)

## RED Phase: Tests First

Write these tests in `src/layer1/unrest_tests.rs` (or a new file) BEFORE any implementation. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::unrest::{MentalState, MentalBreakType, check_mental_break_system};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::{PopAction, ActionType, evaluate_actions_system};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_sleepwalking_trigger() {
        // Arrange: Pop is sleeping (ActionType::SatisfyRest) and highly stressed (Low Morale)
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.1, // Tired
                leisure: 0.1, // Stressed (Low Morale)
            },
            MentalState::Normal,
            PopAction {
                current: ActionType::SatisfyRest, // SLEEPING
                ..Default::default()
            },
        )).id();

        // Act: Run check system
        // Note: Logic in check_mental_break_system needs to be updated to handle Sleepwalking specifically
        world.run_system_once(check_mental_break_system);

        // Assert: MentalState is Broken(Sleepwalk)
        let state = world.get::<MentalState>(pop).unwrap();
        assert_eq!(*state, MentalState::Broken(MentalBreakType::Sleepwalk));
    }

    #[test]
    fn test_sleepwalking_only_triggers_when_sleeping() {
        // Arrange: Pop is WORKING (not sleeping) and highly stressed
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.1,
                leisure: 0.1, // Stressed
            },
            MentalState::Normal,
            PopAction {
                current: ActionType::Work, // NOT SLEEPING
                ..Default::default()
            },
        )).id();

        // Act
        world.run_system_once(check_mental_break_system);

        // Assert: MentalState is NOT Sleepwalk (might be Vandalize/Binge, but not Sleepwalk)
        let state = world.get::<MentalState>(pop).unwrap();
        if let MentalState::Broken(break_type) = state {
            assert_ne!(*break_type, MentalBreakType::Sleepwalk);
        }
    }

    #[test]
    fn test_sleepwalking_action_selection() {
        // Arrange: Pop is already in Sleepwalk state
        let mut world = World::new();
        // Setup necessary resources for utility AI
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs::default(),
            crate::layer1::utility_ai::UtilityWeights::default(),
            MentalState::Broken(MentalBreakType::Sleepwalk),
            PopAction {
                ticks_committed: 100, // Ready to evaluate
                ..Default::default()
            },
        )).id();

        // Act: Run evaluation
        world.run_system_once(evaluate_actions_system);

        // Assert: Action is Sleepwalk
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Sleepwalk);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `MentalBreakType` and `ActionType`

In `src/layer1/unrest.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MentalBreakType {
    Vandalize,
    Binge,
    Daze,
    Sleepwalk, // New
}
```

In `src/layer1/utility_ai/types.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionType {
    // ... existing ...
    Sleepwalk, // New
}
```

### 2. Update `check_mental_break_system`

In `src/layer1/unrest.rs`:
- Add logic to check `PopAction`.
- If `action.current == ActionType::SatisfyRest` AND `needs.morale() < threshold`:
  - Chance to trigger `MentalState::Broken(MentalBreakType::Sleepwalk)`.

```rust
pub fn check_mental_break_system(mut query: Query<(&Needs, &mut MentalState, &PopAction)>) {
    for (needs, mut state, action) in &mut query {
        if *state == MentalState::Normal {
            let morale = needs.morale();
            if morale < 0.2 {
                // Check specifically for sleepwalking trigger
                if action.current == ActionType::SatisfyRest {
                     // In real game, use RNG. For test, deterministic if low morale.
                     *state = MentalState::Broken(MentalBreakType::Sleepwalk);
                } else {
                     // Existing logic for other breaks
                     // *state = MentalState::Broken(MentalBreakType::Vandalize);
                }
            }
        }
    }
}
```

### 3. Update `evaluate_actions_system`

In `src/layer1/utility_ai.rs`:
- Handle `MentalBreakType::Sleepwalk`.
- Select a random valid target on the map (e.g., a Building, Stockpile, or just a random walkable tile).
- Return `ActionType::Sleepwalk`.

```rust
// Inside evaluate_actions_system
MentalBreakType::Sleepwalk => {
    // Pick random target
    // For MVP, just pick a random GridPosition in range?
    // Or iterate `structures_state` and pick one randomly?
    // best_target = random_structure;
    ActionType::Sleepwalk
},
```

### 4. Implement Execution Logic

Create `src/layer1/actions/sleepwalk.rs` (or inside `execution.rs` for MVP).

```rust
pub fn sleepwalk_execution_system(
    mut query: Query<(Entity, &mut PopAction, &mut MentalState, &mut Needs), (With<Pop>, With<AtTarget>)>,
    mut commands: Commands,
) {
    for (entity, mut action, mut state, mut needs) in &mut query {
        if action.current == ActionType::Sleepwalk {
            // "Wake up"
            // Clear broken state
            *state = MentalState::Normal;

            // Return to Idle (will re-evaluate next tick)
            action.current = ActionType::Idle;
            action.ticks_committed = 0;

            // Apply "Grocca" (Groggy/Confused) debuff?
            // For MVP, maybe just reduce Rest slightly to simulate bad sleep?
            needs.rest = (needs.rest - 0.1).max(0.0);

            // Remove AtTarget/MovementTarget components handled by cleanup system usually?
            // Or explicitly remove here if needed.
            commands.entity(entity).remove::<AtTarget>();
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Random Targets**: Instead of just structures, sleepwalkers could target specific things like "The Fridge" (Stockpile with Food) or "The Edge of the Map".
- **Interactions**: Actually *do* something at the target. Drop an item? Toggle a switch?
- **Safety**: Ensure they don't sleepwalk into hazards (fire, vacuum). Or maybe they DO? (Fun but dangerous).
- **Log**: Add a log message "X is sleepwalking...".

## Acceptance Criteria

- [ ] `MentalBreakType::Sleepwalk` exists.
- [ ] Pops only enter Sleepwalk state if they are currently Resting (`SatisfyRest`).
- [ ] Sleepwalking pops move to a target and then wake up.
- [ ] Sleepwalking interrupts Rest recovery (since they stop Resting to Sleepwalk).
- [ ] Tests pass.

## Technical Guidance

- Use `rand::thread_rng()` for target selection in `evaluate_actions_system`.
- Ensure `sleepwalk_execution_system` is added to the `SimulationSchedule`.
- `check_mental_break_system` needs `PopAction` in its query now.
