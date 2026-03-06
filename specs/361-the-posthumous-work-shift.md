# 361 - The Posthumous Work Shift

## 1. Overview
**Layer:** 1
**Fantasy:** Watching the colony run so efficiently that even death is just a temporary delay in the shift schedule.
**Mechanic:** When a pop dies at work, their "digital ghost" (brain scan backup) can be temporarily mapped onto automated chassis or station systems to complete their shift or project before fully decaying.

## 2. Dependencies
- Base Population & Lifecycle mechanics (`003-population-basics`, `062-pop-lifecycle`)
- Work and Job system (`009-job-system`, `066-building-work-ai`)
- Traits/Death system (`034-pop-health`)

## 3. RED Phase: Tests First

```rust
// tests/layer1/tech/posthumous_work_shift_tests.rs

#[test]
fn test_pop_death_spawns_digital_ghost_during_active_shift() {
    // Arrange: Create a Pop actively working on a task, with brain scan tech enabled.
    // Act: Cause the Pop to die (health to 0).
    // Assert: Instead of task immediately failing, a "DigitalGhost" entity is spawned and assigned to the same job.
}

#[test]
fn test_digital_ghost_completes_current_shift_then_decays() {
    // Arrange: Setup a DigitalGhost actively completing a job shift.
    // Act: Advance simulation time to complete the task/shift.
    // Assert: Task is marked completed. The DigitalGhost entity then decays/despawns.
}

#[test]
fn test_digital_ghost_reduces_colony_morale_when_working() {
    // Arrange: A DigitalGhost working alongside living Pops.
    // Act: Evaluate mood of nearby Pops.
    // Assert: Nearby Pops receive a "Haunted Workplace" negative mood modifier.
}

#[test]
fn test_digital_ghost_cannot_be_assigned_new_tasks() {
    // Arrange: An idle DigitalGhost.
    // Act: Try to assign the ghost to a new job via the job assignment system.
    // Assert: Job assignment system rejects the assignment.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/posthumous_work_shift.rs
// Implementation to make tests pass.

use bevy_ecs::prelude::*;
// Add DigitalGhost component and logic to map dying pops to their tasks temporarily.
```

## 5. REFACTOR Phase: Quality & Design
- Integrate ghost spawning directly into the existing `death_system` event flow.
- Ensure the morale impact uses the existing `Needs.leisure` or `MoodModifier` systems.
- Validate that the automated chassis or visual representation handles rendering gracefully.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] Test coverage ≥85% for `posthumous_work_shift` module.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Ghost entities successfully take over exactly one shift/task before dying permanently.
- [ ] Nearby workers experience a morale penalty when a ghost is active.

## 7. Technical Guidance
- `DigitalGhost` should be a distinct component that carries the remaining progress of the original Pop's job.
- Hook into the death processing pipeline: when a `Pop` dies, check if `ActiveJob` exists. If so, spawn `DigitalGhost` with the same `ActiveJob`.
- Make sure `DigitalGhost` doesn't consume food, oxygen, or rest.

## 8. Questions
*Builder: add questions here if spec is unclear.*
