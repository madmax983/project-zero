# 1306: Architectural Superstition

## Overview

Implementing the "Architectural Superstition" mechanic. If multiple negative events (e.g., fires, deaths, low productivity) occur coincidentally near a specific building, pops start associating it with bad luck. A `Cursed` modifier is applied, causing pops to actively avoid working there or passing by, which significantly drops its efficiency. The player is forced to decide whether to demolish and rebuild the functional structure to appease the colony's irrational fears, or force them to use it at a massive mood and efficiency penalty.

## Dependencies

- None explicitly required, though relies on existing building, pop, and event tracking mechanics.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_building_becomes_cursed_after_multiple_negative_events() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_architectural_superstition);

        let building = app.world_mut().spawn((
            Building,
            GridPosition { x: 10, y: 10 },
            NegativeEventHistory::default(),
        )).id();

        // Arrange: Add multiple negative events near the building
        let mut history = app.world_mut().get_mut::<NegativeEventHistory>(building).unwrap();
        history.events.push(NegativeEvent { severity: 5.0, time: 0.0 });
        history.events.push(NegativeEvent { severity: 6.0, time: 1.0 });
        history.events.push(NegativeEvent { severity: 4.0, time: 2.0 });

        // Act
        app.update();

        // Assert: The building should now have the Cursed component
        assert!(app.world().get::<Cursed>(building).is_some());
    }

    #[test]
    fn test_cursed_building_reduces_efficiency_and_mood() {
        let mut app = App::new();
        app.add_systems(Update, apply_cursed_penalties);

        let building = app.world_mut().spawn((
            Building,
            Cursed,
            Efficiency(1.0),
        )).id();

        // Act
        app.update();

        // Assert: Efficiency should be reduced
        let efficiency = app.world().get::<Efficiency>(building).unwrap();
        assert!(efficiency.0 < 1.0);
    }

    #[test]
    fn test_pops_avoid_cursed_buildings_in_pathfinding() {
        // Test that pathfinding cost is increased for cursed building tiles
        let mut app = App::new();
        // Setup grid and pathfinding...

        let cursed_building = app.world_mut().spawn((
            Building,
            GridPosition { x: 5, y: 5 },
            Cursed,
        )).id();

        // Act: Request a path that would normally go through (5,5)
        // Assert: The generated path avoids (5,5) due to increased cost
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building;

pub struct NegativeEvent {
    pub severity: f32,
    pub time: f32,
}

#[derive(Component, Default)]
pub struct NegativeEventHistory {
    pub events: Vec<NegativeEvent>,
}

#[derive(Component)]
pub struct Cursed;

#[derive(Component)]
pub struct Efficiency(pub f32);

pub fn evaluate_architectural_superstition(
    mut commands: Commands,
    query: Query<(Entity, &NegativeEventHistory), Without<Cursed>>,
) {
    for (entity, history) in query.iter() {
        // Simple threshold: 3 or more negative events
        if history.events.len() >= 3 {
            let total_severity: f32 = history.events.iter().map(|e| e.severity).sum();
            if total_severity >= 10.0 {
                commands.entity(entity).insert(Cursed);
            }
        }
    }
}

pub fn apply_cursed_penalties(
    mut query: Query<&mut Efficiency, With<Cursed>>,
) {
    for mut efficiency in query.iter_mut() {
        // Significantly drop efficiency
        efficiency.0 *= 0.5;
    }
}
```

## REFACTOR Phase: Quality & Design

- **Event Pruning**: The `NegativeEventHistory` will grow indefinitely. Implement a mechanism to cull events that are too old so buildings aren't cursed for events that happened generations ago.
- **Gradual Cursing**: Instead of a boolean `Cursed` state, consider a sliding scale of "Superstition Level" that gradually affects efficiency and pathfinding cost.
- **Player Mitigation**: Implement ways for the player to "cleans" a building (e.g., through expensive ceremonies or public relations campaigns) without needing full demolition.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Specific feature behavior verified: Multiple negative events apply the `Cursed` status, reducing efficiency and altering pop pathfinding.

## Technical Guidance

### Components

- `NegativeEventHistory`: Add to buildings to track nearby negative events.
- `Cursed`: Marker component (or value component) applied to buildings meeting the superstition threshold.

### Systems

- `track_negative_events`: Listen for negative events (deaths, fires, etc.) and record them in the nearest building's history.
- `evaluate_architectural_superstition`: Periodically check histories and apply `Cursed` status.
- `apply_cursed_penalties`: Apply efficiency modifiers to cursed buildings.
- Update pathfinding logic to incorporate an avoidance cost for tiles adjacent to cursed buildings.

### Integration Points

- Needs to integrate with the existing event system to capture negative occurrences.
- Pathfinding weights (likely A* in `src/layer1/map.rs` or `terrain.rs`) need to be dynamically adjusted based on the `Cursed` status of nearby buildings.
- UI should visually indicate the cursed status and the source of the pops' fears.

## Questions

*Builder: add questions here if spec is unclear.*
