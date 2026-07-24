# 1329: Memorial Subroutines

## Overview
The automated systems of a colony remember the dead better than the living do, creating haunting algorithmic shrines. When a Pop with a high 'Importance' score dies, the Utility AI algorithms governing automated systems (like power distribution or drone pathing) develop a "Memorial Subroutine". These systems will irrationally prioritize areas the dead Pop frequented, keeping their old workspace perfectly heated and lit while plunging vital new sectors into darkness.

## Dependencies
- `004` — Pop Entity
- `009` — Job System
- `010` — Power Grid

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::jobs::{Job, JobType};
    use crate::layer1::memorial_subroutines::{MemorialSubroutine, MemorialTarget, spawn_memorial_subroutine_on_death};
    use crate::layer1::social::importance::Importance;
    use crate::layer1::death::PopDiedEvent;
    use crate::layer1::core::map::GridPosition;

    #[test]
    fn test_high_importance_pop_death_spawns_subroutine() {
        let mut app = App::new();
        app.add_event::<PopDiedEvent>();
        app.add_systems(Update, spawn_memorial_subroutine_on_death);

        // Spawn a pop with high importance
        let entity = app.world_mut().spawn((
            Pop::new(),
            Importance { value: 90.0 }, // High importance
            Job { job_type: JobType::Research },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Trigger death event
        app.world_mut().send_event(PopDiedEvent { pop_entity: entity });

        app.update();

        // Check if a memorial subroutine was spawned targeting the pop's last position
        let mut query = app.world_mut().query::<&MemorialSubroutine>();
        let subroutines: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(subroutines.len(), 1, "A MemorialSubroutine should be spawned for a high importance pop.");
        assert_eq!(subroutines[0].target_pos, GridPosition { x: 10, y: 10 }, "The subroutine should target the pop's last position.");
    }

    #[test]
    fn test_low_importance_pop_death_no_subroutine() {
        let mut app = App::new();
        app.add_event::<PopDiedEvent>();
        app.add_systems(Update, spawn_memorial_subroutine_on_death);

        // Spawn a pop with low importance
        let entity = app.world_mut().spawn((
            Pop::new(),
            Importance { value: 10.0 }, // Low importance
            Job { job_type: JobType::Mining },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Trigger death event
        app.world_mut().send_event(PopDiedEvent { pop_entity: entity });

        app.update();

        let mut query = app.world_mut().query::<&MemorialSubroutine>();
        let subroutines: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(subroutines.len(), 0, "No MemorialSubroutine should be spawned for a low importance pop.");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::death::PopDiedEvent;
use crate::layer1::social::importance::Importance;
use crate::layer1::core::map::GridPosition;

#[derive(Component, Debug, Clone)]
pub struct MemorialSubroutine {
    pub target_pos: GridPosition,
    pub priority_boost: f32,
}

#[derive(Component)]
pub struct MemorialTarget;

pub fn spawn_memorial_subroutine_on_death(
    mut events: EventReader<PopDiedEvent>,
    query: Query<(&Importance, &GridPosition)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok((importance, pos)) = query.get(event.pop_entity) {
            // Threshold for spawning a memorial subroutine
            if importance.value >= 80.0 {
                commands.spawn(MemorialSubroutine {
                    target_pos: *pos,
                    priority_boost: 5.0, // Significant boost to power/resource allocation
                });
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Power Grid Override**: Integrate the `MemorialSubroutine` into the `power_grid_system`. Areas targeted by the subroutine should receive power *first*, even if they have no active consumers, acting as a sink.
- **Drone Pathing**: If a `MemorialSubroutine` exists, automated drones might occasionally path to `target_pos` and idle there briefly as an "inspection" routine.
- **Purge Action**: Provide a way for the player to "purge" these subroutines. This could require a specific `JobType` (like IT/Engineer) or cost resources.

## Acceptance Criteria
- [ ] Tests in RED phase pass.
- [ ] `MemorialSubroutine` component is spawned when a Pop with `Importance >= 80.0` dies.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage ≥85% for new code.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance
- Listen to `PopDiedEvent`. If the dead pop had a high `Importance` score, capture their last known `GridPosition` or their primary workplace `GridPosition` and spawn a global entity with the `MemorialSubroutine` component.
- The `power_grid_system` should query for `MemorialSubroutine` entities and artificially inflate the power priority of the `target_pos`.

## Questions
*Builder: add questions here if spec is unclear.*
