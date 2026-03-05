# Specification: The Phantom Shift (Layer 1)

## 1. Overview
Under conditions of extreme systemic inefficiency (e.g., massive job backlogs, neglected infrastructure), a "Phantom Shift" emerges. Pops with the "Fringe" cultural tag or high desperation secretly work during the night cycle. They fix things and complete jobs but use up colony resources without logging them and slowly build an invisible, untaxable shadow economy. This creates a tension between free automated labor and the loss of economic transparency.

## 2. Dependencies
- `009` Job System
- `016` Utility AI System
- `273` The Feral Outpost (for the "Fringe" cultural tag, or rely on a fallback like high Unrest/Stress if not fully integrated).
- `065` Day/Night Cycle

## 3. RED Phase: Tests First

```rust
// src/layer1/phantom_shift_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::culture::CulturalTag;
    use crate::layer1::time::DayNightCycle;
    use crate::layer1::utility_ai::{UtilityAIBuffer, ActionType};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems
        world.insert_resource(DayNightCycle { is_night: true, ..Default::default() });
        world.insert_resource(PhantomShiftConfig { inefficiency_threshold: 50.0 });
        world.insert_resource(ColonyInefficiencyTracker { current_backlog_score: 100.0 });
        world
    }

    #[test]
    fn test_phantom_shift_activation_at_night() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            CulturalTag::Fringe,
            UtilityAIBuffer::default(),
        )).id();

        world.run_system_once(evaluate_phantom_shift_utility);

        let buffer = world.get::<UtilityAIBuffer>(pop).unwrap();
        // Should evaluate high utility for PhantomWork during night with high inefficiency
        assert!(buffer.actions.iter().any(|a| matches!(a.action_type, ActionType::PhantomWork)));
    }

    #[test]
    fn test_phantom_shift_does_not_activate_during_day() {
        let mut world = setup_world();
        world.resource_mut::<DayNightCycle>().is_night = false;

        let pop = world.spawn((
            Pop,
            CulturalTag::Fringe,
            UtilityAIBuffer::default(),
        )).id();

        world.run_system_once(evaluate_phantom_shift_utility);

        let buffer = world.get::<UtilityAIBuffer>(pop).unwrap();
        // Should not evaluate PhantomWork during the day
        assert!(!buffer.actions.iter().any(|a| matches!(a.action_type, ActionType::PhantomWork)));
    }

    #[test]
    fn test_phantom_shift_consumes_resources_silently() {
        let mut world = setup_world();
        // Setup phantom work execution environment...
        // Assert that global resource trackers drift from actual stockpile counts.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/phantom_shift.rs

use bevy_ecs::prelude::*;
use crate::layer1::time::DayNightCycle;
use crate::layer1::culture::CulturalTag;
use crate::layer1::utility_ai::{ActionType, UtilityAIBuffer};

#[derive(Resource)]
pub struct PhantomShiftConfig {
    pub inefficiency_threshold: f32,
}

impl Default for PhantomShiftConfig {
    fn default() -> Self {
        Self { inefficiency_threshold: 80.0 }
    }
}

#[derive(Resource, Default)]
pub struct ColonyInefficiencyTracker {
    pub current_backlog_score: f32,
}

pub fn evaluate_phantom_shift_utility(
    cycle: Res<DayNightCycle>,
    tracker: Res<ColonyInefficiencyTracker>,
    config: Res<PhantomShiftConfig>,
    mut query: Query<(&CulturalTag, &mut UtilityAIBuffer), With<crate::layer1::pop::Pop>>,
) {
    if !cycle.is_night || tracker.current_backlog_score < config.inefficiency_threshold {
        return;
    }

    for (tag, mut buffer) in query.iter_mut() {
        if *tag == CulturalTag::Fringe {
            // Phantom work overrides rest with a moderate/high utility
            buffer.add_action(ActionType::PhantomWork, 8.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Inefficiency Metric:** Define `ColonyInefficiencyTracker` properly by analyzing the number of unfulfilled job designations compared to available workers.
- **Resource Siphoning:** When `ActionType::PhantomWork` executes, it should bypass standard logging mechanisms so the UI does not immediately reflect the consumed resources, creating "Shadow Economy" drift.
- **Discovery:** Add a mechanic where the "Inspector" or player can uncover phantom work, generating an event that forces a choice: crack down or ignore.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] `ActionType::PhantomWork` is added and integrated into the global `UtilityAIBuffer`.
- [ ] Phantom Shift only triggers at night for Fringe pops when inefficiency is high.

## 7. Technical Guidance
- **ActionType:** Add `PhantomWork` to `src/layer1/utility_types.rs`.
- **System Placement:** Register `evaluate_phantom_shift_utility` in the AI evaluation schedule alongside other utility scorers.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect: I will answer your questions as they come up.*
