# 088: Omens & Taboos

## Overview

The colony is a superstitious place. When negative events occur (e.g., a mine collapse), the colonists may associate the recent activity (Mining) with bad luck, forming a **Taboo**.

A **Taboo** is a temporary colony-wide aversion to a specific `ActionType`.
- **Formation**: Triggered by events like `StructureCollapsed`.
- **Effect**:
    1.  **Reluctance**: Pops are less likely to choose the Taboo action (utility penalty).
    2.  **Stress**: If a Pop *does* perform the action (because it's critical), they suffer increased Stress (Leisure decay).
- **Duration**: Taboos fade over time (e.g., 2-3 days).

This adds friction to the simulation. A series of accidents might paralyze your industry not because of physics, but because of fear.

## Dependencies

- `016` — Utility AI System (ActionTypes, Utility Scoring)
- `071` — Structural Integrity (Source of `StructureCollapsed` event)
- `031` — Pop Morale (Stress/Leisure mechanics)

## RED Phase: Tests First

Write these tests in `src/layer1/taboo_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
    use crate::layer1::structural_integrity::StructureCollapsed;
    use crate::layer1::taboo::{TabooState, taboo_event_system, apply_taboo_stress_system, evaluate_taboo_penalty};
    use crate::layer1::needs::Needs;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_taboo_state_default() {
        let state = TabooState::default();
        assert!(state.active_taboos.is_empty());
    }

    #[test]
    fn test_structure_collapse_creates_taboo() {
        let mut world = World::new();
        world.insert_resource(TabooState::default());
        world.insert_resource(SimulationTime::default());

        // Register event
        world.add_event::<StructureCollapsed>();

        // Trigger event
        world.send_event(StructureCollapsed {
            pos: GridPosition { x: 0, y: 0 }
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(taboo_event_system);
        schedule.run(&mut world);

        let state = world.resource::<TabooState>();
        // Work action (which includes mining/building) should be taboo
        assert!(state.is_taboo(ActionType::Work));
        assert!(state.get_duration(ActionType::Work) > 0);
    }

    #[test]
    fn test_taboo_decays_over_time() {
        let mut world = World::new();
        let mut state = TabooState::default();

        // Add taboo manually
        state.add_taboo(ActionType::Work, 10); // 10 ticks duration
        world.insert_resource(state);
        world.insert_resource(SimulationTime { tick: 0, ..Default::default() });

        // Advance time
        use crate::layer1::taboo::update_taboo_duration_system;
        let mut schedule = Schedule::default();
        schedule.add_systems(update_taboo_duration_system);

        schedule.run(&mut world); // Tick 0 -> 1? (Depending on impl)

        // Manually simulate time passing
        let mut state = world.resource_mut::<TabooState>();
        state.decay(1);

        assert_eq!(state.get_duration(ActionType::Work), 9);

        state.decay(9);
        assert!(!state.is_taboo(ActionType::Work));
    }

    #[test]
    fn test_taboo_applies_stress_to_worker() {
        let mut world = World::new();
        let mut state = TabooState::default();
        state.add_taboo(ActionType::Work, 100);
        world.insert_resource(state);

        let pop = world.spawn((
            Pop,
            Needs { leisure: 1.0, ..Default::default() },
            PopAction { current: ActionType::Work, ..Default::default() },
        )).id();

        // Run stress system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_taboo_stress_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        // Leisure should drop (stress increase)
        assert!(needs.leisure < 1.0);
    }

    #[test]
    fn test_evaluate_taboo_penalty() {
        let mut state = TabooState::default();
        state.add_taboo(ActionType::Work, 100);

        let penalty = evaluate_taboo_penalty(ActionType::Work, &state);
        let normal = evaluate_taboo_penalty(ActionType::Idle, &state);

        assert!(penalty < 0.0); // Should return a negative modifier
        assert_eq!(normal, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `StructureCollapsed` Event

In `src/layer1/structural_integrity.rs`:

```rust
#[derive(Event, Debug, Clone)]
pub struct StructureCollapsed {
    pub pos: GridPosition,
}
```

Update `apply_collapse` to emit this event instead of just logging.

### 2. Define `TabooState` Resource

Create `src/layer1/taboo.rs`:

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::utility_ai::ActionType;

#[derive(Resource, Default, Debug)]
pub struct TabooState {
    // Map ActionType -> Remaining Duration (ticks)
    pub active_taboos: HashMap<ActionType, u32>,
}

impl TabooState {
    pub fn add_taboo(&mut self, action: ActionType, duration: u32) {
        // Overwrite or extend? Let's take max duration.
        let current = self.active_taboos.entry(action).or_insert(0);
        *current = (*current).max(duration);
    }

    pub fn is_taboo(&self, action: ActionType) -> bool {
        self.active_taboos.contains_key(&action)
    }

    pub fn get_duration(&self, action: ActionType) -> u32 {
        *self.active_taboos.get(&action).unwrap_or(&0)
    }

    pub fn decay(&mut self, amount: u32) {
        self.active_taboos.retain(|_, duration| {
            *duration = duration.saturating_sub(amount);
            *duration > 0
        });
    }
}
```

### 3. Implement Systems

In `src/layer1/taboo.rs`:

```rust
use crate::layer1::structural_integrity::StructureCollapsed;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::PopAction;
use crate::shared::log::MessageLog;

pub fn taboo_event_system(
    mut events: EventReader<StructureCollapsed>,
    mut state: ResMut<TabooState>,
    mut log: ResMut<MessageLog>,
) {
    for _event in events.read() {
        // Collapse -> Fear of Work (Construction/Mining)
        // Duration: 1 Day (approx 100 ticks for test, 2400 real)
        state.add_taboo(ActionType::Work, 500);
        log.add("The collapse has caused a fear of labor!".to_string());
    }
}

pub fn update_taboo_duration_system(mut state: ResMut<TabooState>) {
    state.decay(1);
}

pub fn apply_taboo_stress_system(
    state: Res<TabooState>,
    mut query: Query<(&PopAction, &mut Needs)>,
) {
    if state.active_taboos.is_empty() {
        return;
    }

    for (action, mut needs) in &mut query {
        if state.is_taboo(action.current) {
            // Apply stress (reduce leisure)
            // 0.005 per tick is significant (1.0 in 200 ticks)
            needs.leisure = (needs.leisure - 0.005).max(0.0);
        }
    }
}

// Helper for Utility AI
pub fn evaluate_taboo_penalty(action: ActionType, state: &TabooState) -> f32 {
    if state.is_taboo(action) {
        -0.3 // Significant penalty to utility score
    } else {
        0.0
    }
}
```

### 4. Integration with Utility AI

Modify `evaluate_work` and other evaluators in `src/layer1/actions/*.rs` (or `utility_ai.rs` if central) to call `evaluate_taboo_penalty`.

Example in `evaluate_work`:

```rust
// In evaluate_work args, add `taboo_state: &TabooState`
// ...
let mut utility = ...;
utility += evaluate_taboo_penalty(ActionType::Work, taboo_state);
// ...
```

## REFACTOR Phase: Quality & Design

- **Visual Feedback**: Add a "fear" icon above Pops performing taboo actions.
- **Dynamic Action Mapping**: Instead of hardcoding `StructureCollapsed -> Work`, use a `TabooRules` resource to map Events to Actions.
- **Omen System**: Positive events (e.g., `BumperHarvest`) could create "Blessings" (positive utility modifiers).
- **Log Spam**: Ensure logs don't spam if multiple tiles collapse at once.

## Acceptance Criteria

- [ ] `TabooState` tracks active taboos.
- [ ] `StructureCollapsed` event triggers a `Work` taboo.
- [ ] Pops performing `Work` during a taboo lose `Leisure` (Stress).
- [ ] Utility AI scores for `Work` are reduced when taboo is active.
- [ ] Taboos expire after a duration.
- [ ] All new tests pass.

## Technical Guidance

- Be careful with `EventReader` in `taboo_event_system`. Ensure it consumes events so they don't persist.
- `StructureCollapsed` logic in `structural_integrity.rs` must be refactored to *emit* the event. Currently, it likely just modifies terrain/logs.
- Make sure to register the `StructureCollapsed` event in the app build.
