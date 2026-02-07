# 050: Civil Unrest

## Overview

When morale drops dangerously low, Pops suffer mental breaks, becoming uncontrollable and destructive.
This forces players to manage morale proactively, not just for efficiency but for survival.
It introduces the `MentalState` component and overrides standard Utility AI logic during breakdowns.

## Dependencies

- `031` — Pop Morale (Needs system)
- `045` — Structure Durability (Target for Vandalism)
- `016` — Utility AI (Action evaluation override)
- `022` — Stockpiles (Target for Binge)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/unrest_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, MentalState, MentalBreakType};
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_ai::{ActionType, PopAction, evaluate_actions_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::Structure; // For HP
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        // Insert required resources for Utility AI
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_mental_state_default() {
        let state = MentalState::default();
        assert_eq!(state, MentalState::Normal);
    }

    #[test]
    fn test_check_break_risk_triggers_break() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 0.1,
                rest: 0.1,
                leisure: 0.1,
            }, // Very low morale (~0.1)
            MentalState::Normal,
        )).id();

        // Run check system (to be implemented)
        crate::layer1::unrest::check_mental_break_system(&mut world);

        let state = world.get::<MentalState>(pop).unwrap();
        // Should have transitioned to Broken
        assert!(matches!(state, MentalState::Broken(_)));
    }

    #[test]
    fn test_broken_state_overrides_utility_ai() {
        let mut world = setup_world();

        // Pop is Broken(Vandalize)
        let pop = world.spawn((
            Pop,
            Needs::default(), // Even with full needs, if broken, should act broken until recovered
            MentalState::Broken(MentalBreakType::Vandalize),
            PopAction::default(),
            GridPosition { x: 0, y: 0 },
            crate::layer1::utility_ai::UtilityWeights::default(),
        )).id();

        // Add a building target
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
            Structure { hp: 100.0, max_hp: 100.0, ..Default::default() },
        ));

        // Run utility AI
        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Vandalize);
    }

    #[test]
    fn test_vandalize_damages_building() {
        let mut world = setup_world();

        let building = world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
            Structure { hp: 100.0, max_hp: 100.0, ..Default::default() },
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MentalState::Broken(MentalBreakType::Vandalize),
            // Assume perform_vandalize_system uses PopAction or similar
        )).id();

        // Manually trigger damage logic (simulating system behavior)
        crate::layer1::unrest::perform_vandalize_logic(&mut world, pop, building);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.hp < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `MentalState` and Enums

```rust
// src/layer1/pop.rs or src/layer1/unrest.rs

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum MentalState {
    Normal,
    Broken(MentalBreakType),
}

impl Default for MentalState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MentalBreakType {
    Vandalize,
    Binge,
    Daze,
}
```

### 2. Update `ActionType`

```rust
// src/layer1/utility_ai/types.rs

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActionType {
    // ... existing ...
    Vandalize, // New
    Binge,     // New
    Daze,      // New
}

impl ActionType {
    // Update total count
    pub const COUNT: usize = 13; // 10 existing + 3 new

    pub const fn as_index(self) -> usize {
        match self {
            // ... existing ...
            Self::Vandalize => 10,
            Self::Binge => 11,
            Self::Daze => 12,
        }
    }
}
```

### 3. Implement `check_mental_break_system`

```rust
// src/layer1/unrest.rs

pub fn check_mental_break_system(mut query: Query<(&Needs, &mut MentalState)>) {
    for (needs, mut state) in &mut query {
        if *state == MentalState::Normal {
            let morale = needs.morale();
            if morale < 0.15 {
                // Random chance or deterministic for now.
                // In a real system, use RNG.
                *state = MentalState::Broken(MentalBreakType::Vandalize); // Simplified for Green
            }
        }
        // Logic to recover from break (e.g. time passed, needs improved) is needed too.
    }
}
```

### 4. Update `evaluate_actions_system`

Modify `src/layer1/utility_ai.rs` to prioritize breakdown actions.

```rust
// In evaluate_actions_system loop:

let mental_state = world.get::<MentalState>(pop_entity).unwrap_or(&MentalState::Normal);

match mental_state {
    MentalState::Broken(break_type) => {
        // Force specific action based on break type
        let (action_type, utility, target) = match break_type {
            MentalBreakType::Vandalize => {
                // Find random building to destroy
                // Return (ActionType::Vandalize, 100.0, Some(target))
                (ActionType::Vandalize, 100.0, None) // Placeholder
            },
            MentalBreakType::Binge => (ActionType::Binge, 100.0, None),
            MentalBreakType::Daze => (ActionType::Daze, 100.0, None),
        };

        utilities.push((action_type, utility, target));
    },
    MentalState::Normal => {
        // ... existing logic ...
    }
}
```

## REFACTOR Phase: Quality & Design

- **GPU Shader Updates**:
    - **CRITICAL**: You MUST update `src/gpu/buffers.rs` to match the new `ActionType::COUNT`.
    - **CRITICAL**: You MUST update `src/gpu/shaders/evaluate.wgsl` if it hardcodes the action array size or indices.
- **Breakdown Variety**: Implement weighted random selection for break types based on traits (e.g., `Pyromaniac` -> `FireStarting` (future), `Glutton` -> `Binge`).
- **Catharsis**: Add a `Catharsis` modifier to `Needs` after a break ends to prevent loops.
- **Recovery**: Implement `recover_mental_break_system` that checks if needs are met or time passed.

## Acceptance Criteria

- [ ] `MentalState` component exists.
- [ ] Low morale (< 0.15) triggers `MentalState::Broken`.
- [ ] `ActionType` updated with `Vandalize`, `Binge`, `Daze`.
- [ ] `evaluate_actions_system` respects `MentalState` (overrides normal utility).
- [ ] `Vandalize` action reduces building HP.
- [ ] GPU buffers and shaders updated to reflect new `ActionType` count.
- [ ] Tests pass.

## Technical Guidance

- When modifying `ActionType`, remember it's used as an index in `UtilityWeights`. The arrays `action_success_count` will grow.
- `GpuPopInput` likely has padding. Ensure alignment is preserved when adding fields or changing constants.
- Vandalism should check `Structure` component for valid targets. Don't vandalize `Indestructible` things (if any).
