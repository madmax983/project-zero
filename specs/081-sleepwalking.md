# 081: Sleepwalking

## Overview

Stressed Pops (low Morale) have a chance to enter a `Sleepwalking` state instead of performing the `SatisfyRest` action. While sleepwalking, they wander randomly, do not recover Rest, and may end up in dangerous or inconvenient locations. This adds a layer of emergent storytelling where stress manifests as subconscious behavior.

## Dependencies

- `005` — Pop Needs (Rest, Morale)
- `050` — Civil Unrest (MentalState, MentalBreakType)
- `016` — Utility AI (ActionType)

## RED Phase: Tests First

Write these tests in `src/layer1/sleepwalking_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, MentalState, MentalBreakType};
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_ai::{ActionType, PopAction, evaluate_actions_system};
    use crate::layer1::unrest::check_mental_break_system;
    use crate::layer1::map::GridPosition;
    use crate::layer1::sleepwalking::SleepwalkingConfig;

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(SleepwalkingConfig { chance: 1.0 }); // Deterministic testing
        world
    }

    #[test]
    fn test_sleepwalking_trigger() {
        let mut world = setup_world();

        // Pop attempting to rest with low morale
        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.1, // Needs rest
                leisure: 0.1, // Low morale
            },
            MentalState::Normal,
            PopAction {
                current: ActionType::SatisfyRest, // Currently resting
                ..Default::default()
            },
        )).id();

        // Run trigger system (new system)
        crate::layer1::sleepwalking::check_sleepwalking_start_system(&mut world);

        // Check if state changed
        let state = world.get::<MentalState>(pop).unwrap();
        // Should be Broken(Sleepwalking) because chance is 1.0
        assert!(matches!(state, MentalState::Broken(MentalBreakType::Sleepwalking)));
    }

    #[test]
    fn test_sleepwalking_overrides_action() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Needs::default(),
            MentalState::Broken(MentalBreakType::Sleepwalking),
            PopAction::default(),
            GridPosition { x: 0, y: 0 },
            crate::layer1::utility_ai::UtilityWeights::default(),
        )).id();

        // Run Utility AI
        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Sleepwalking);
    }

    #[test]
    fn test_sleepwalking_movement() {
        use crate::layer1::execution::{process_start_plan_system, movement_system};
        use crate::layer1::utility_ai::StartPlan;

        let mut world = setup_world();
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            MentalState::Broken(MentalBreakType::Sleepwalking),
            PopAction {
                current: ActionType::Sleepwalking,
                ..Default::default()
            },
            crate::layer1::utility_ai::UtilityWeights::default(),
            // Sleepwalkers need a target to move to.
            // evaluate_actions_system should pick a random target.
        )).id();

        // Run AI to generate StartPlan
        evaluate_actions_system(&mut world);

        let plan = world.get::<StartPlan>(pop).unwrap();
        assert!(plan.target.is_some());
        assert_ne!(plan.target.unwrap(), pop); // Should target somewhere else

        // Run movement logic
        world.run_system_once(process_start_plan_system).unwrap();
        world.run_system_once(movement_system).unwrap();

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert!(*pos != GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_sleepwalking_recovery() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            MentalState::Broken(MentalBreakType::Sleepwalking),
            crate::layer1::sleepwalking::SleepwalkTimer(1), // 1 tick remaining
        )).id();

        // Run recovery system
        crate::layer1::sleepwalking::sleepwalk_end_system(&mut world);

        let state = world.get::<MentalState>(pop).unwrap();
        assert_eq!(*state, MentalState::Normal);
        assert!(world.get::<crate::layer1::sleepwalking::SleepwalkTimer>(pop).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

```rust
// src/layer1/unrest.rs
pub enum MentalBreakType {
    // ... existing ...
    Sleepwalking, // New
}

// src/layer1/utility_ai/types.rs
pub enum ActionType {
    // ... existing ...
    Sleepwalking, // New
}

impl ActionType {
    pub const COUNT: usize = 14; // Increment this!
    pub const fn as_index(self) -> usize {
        match self {
            // ... existing ...
            Self::Sleepwalking => 13,
        }
    }
}
```

### 2. Define Components and Config

```rust
// src/layer1/sleepwalking.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, MentalState, MentalBreakType};
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::{PopAction, ActionType};
use rand::Rng;

#[derive(Component, Debug, Clone, Copy)]
pub struct SleepwalkTimer(pub u32); // Ticks remaining

#[derive(Resource, Debug, Clone)]
pub struct SleepwalkingConfig {
    pub chance: f64,
}

impl Default for SleepwalkingConfig {
    fn default() -> Self {
        Self { chance: 0.01 }
    }
}
```

### 3. Implement Systems

```rust
// src/layer1/sleepwalking.rs

pub fn check_sleepwalking_start_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs, &PopAction, &mut MentalState), Without<SleepwalkTimer>>,
    config: Option<Res<SleepwalkingConfig>>,
) {
    let mut rng = rand::thread_rng();
    let chance = config.map_or(0.01, |c| c.chance);

    for (entity, needs, action, mut state) in &mut query {
        // Trigger condition: Trying to rest AND low morale
        if action.current == ActionType::SatisfyRest && needs.morale() < 0.25 {
            if rng.gen_bool(chance) {
                *state = MentalState::Broken(MentalBreakType::Sleepwalking);
                commands.entity(entity).insert(SleepwalkTimer(100)); // Sleepwalk for 100 ticks
            }
        }
    }
}

pub fn sleepwalk_end_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MentalState, &mut SleepwalkTimer)>,
) {
    for (entity, mut state, mut timer) in &mut query {
        if matches!(*state, MentalState::Broken(MentalBreakType::Sleepwalking)) {
            if timer.0 > 0 {
                timer.0 -= 1;
            } else {
                *state = MentalState::Normal;
                commands.entity(entity).remove::<SleepwalkTimer>();
            }
        } else {
            // Safety cleanup if state changed externally
            commands.entity(entity).remove::<SleepwalkTimer>();
        }
    }
}
```

### 4. Update AI Logic

```rust
// src/layer1/utility_ai.rs

// Inside evaluate_actions_system match mental_state:
MentalBreakType::Sleepwalking => {
    // Pick random target on map
    // For MVP Green, assume execution system handles random movement if target is None or Self
    // Or pick a random walkable tile nearby.
    (ActionType::Sleepwalking, 100.0, None)
},
```

### 5. Execution Logic

Update `process_start_plan_system` in `src/layer1/execution.rs` (or `sleepwalking.rs`) to handle `ActionType::Sleepwalking`.

```rust
// If action is Sleepwalking and target is None:
// Pick a random walkable tile within radius 10.
// Set MovementTarget to that tile.
```

## REFACTOR Phase: Quality & Design

- **GPU Buffers**: Update `src/gpu/buffers.rs` `GpuPopInput` padding/alignment if `ActionType::COUNT` changes size significantly (it's array size, so yes).
- **Shader**: Update `evaluate.wgsl` with new action index.
- **Safety**: Ensure sleepwalkers don't walk into fire or vacuum (unless that's the point?).
- **Feedback**: Add "Zzz" particle effect or icon.

## Acceptance Criteria

- [ ] `Sleepwalking` added to `MentalBreakType` and `ActionType`.
- [ ] `SleepwalkingConfig` resource created.
- [ ] Stressed pops (morale < 0.25) have chance to sleepwalk when resting.
- [ ] Sleepwalkers move randomly and do not recover Rest.
- [ ] Sleepwalking ends after fixed duration.
- [ ] Tests pass (deterministic with config).

## Technical Guidance

- `ActionType::COUNT` MUST match `src/gpu/buffers.rs`.
- `SleepwalkTimer` should be decremented in a system that runs every tick.
- Ensure `check_sleepwalking_start_system` runs *before* `evaluate_actions_system`? Or after? If after, it takes effect next tick.
