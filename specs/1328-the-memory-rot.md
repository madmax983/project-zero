# 1328: The Memory Rot

## Overview

The psychological burden of remembering the galaxy's history becomes a physical hazard. Pops assigned to 'Archive' jobs accumulate a 'Memory Rot' debuff over time as they process ancient, fragmented data. If the debuff maxes out, they enter a fugue state, abandoning their needs and obsessively engraving cryptic warnings on nearby structures until they are treated in a Medical Bay.

## Dependencies

- `004` — Pop Entity
- `009` — Job System

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::jobs::{Job, JobType};
    use crate::layer1::memory_rot::{MemoryRot, FugueState, process_memory_rot};

    #[test]
    fn test_memory_rot_accumulation() {
        let mut world = World::new();
        // Pop working as an Archivist
        let entity = world.spawn((
            Pop::new(),
            Job { job_type: JobType::Archive },
            MemoryRot { amount: 0.0 }
        )).id();

        // Run the system once
        process_memory_rot(&mut world);

        let rot = world.get::<MemoryRot>(entity).unwrap();
        assert!(rot.amount > 0.0, "Memory Rot should accumulate for Archive jobs");
    }

    #[test]
    fn test_fugue_state_trigger() {
        let mut world = World::new();
        let max_rot = 100.0;
        let entity = world.spawn((
            Pop::new(),
            Job { job_type: JobType::Archive },
            MemoryRot { amount: max_rot - 1.0 }
        )).id();

        process_memory_rot(&mut world);

        // Pop should now have maxed out rot and entered a fugue state
        assert!(world.get::<FugueState>(entity).is_some(), "Pop should enter FugueState when MemoryRot maxes out");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::jobs::{Job, JobType};

#[derive(Component, Default, Debug)]
pub struct MemoryRot {
    pub amount: f32,
}

#[derive(Component, Default, Debug)]
pub struct FugueState;

pub fn process_memory_rot(world: &mut World) {
    let mut query = world.query::<(Entity, &Job, &mut MemoryRot)>();
    let mut to_add_fugue = Vec::new();

    for (entity, job, mut rot) in query.iter_mut(world) {
        if job.job_type == JobType::Archive {
            rot.amount += 5.0;

            if rot.amount >= 100.0 {
                to_add_fugue.push(entity);
            }
        }
    }

    for entity in to_add_fugue {
        world.entity_mut(entity).insert(FugueState);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Fugue State AI Override**: The `FugueState` component must be integrated with the Utility AI system. It should override normal need satisfaction (eating, sleeping) and force the pop to find the nearest structure to "engrave".
- **Vandalism Mechanics**: Integrate the engraving action with the `Vandalized` component (from Cultural Vandalism spec if available) or apply a generic damage/morale debuff to the structure.
- **Medical Treatment**: Ensure Medical Bay jobs or Doctor pops have a task defined to remove `FugueState` and reset `MemoryRot`.

## Acceptance Criteria

- [ ] `MemoryRot` and `FugueState` components exist.
- [ ] Tests in RED phase pass.
- [ ] `MemoryRot` accumulates correctly when pops are assigned to Archive jobs.
- [ ] `FugueState` is applied when `MemoryRot` reaches the maximum threshold.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage ≥85% for new code.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance

- Ensure the accumulation rate of `MemoryRot` can be balanced easily (perhaps use a resource for settings instead of hardcoding `5.0`).
- The `FugueState` should probably also remove the `Job` component or interrupt the current task to ensure the pop immediately stops archiving and begins wandering/vandalizing.
- Work closely with existing `UtilityAI` mechanisms to prioritize the erratic behavior cleanly without breaking the core loop.

## Questions
*Builder: add questions here if spec is unclear.*
