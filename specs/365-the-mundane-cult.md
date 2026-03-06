# 365 - The Mundane Cult

## 1. Overview
**Layer:** 1
**Fantasy:** Watching a religion form not around gods or space anomalies, but around the most boring, repetitive aspects of colony life.
**Mechanic:** Pops performing the same low-tier job (e.g., hauling dirt, cleaning filters) for extended periods have a chance to form a cult venerating that specific action. They will demand dedicated shrines (e.g., "The Holy Air Filter") and refuse to do any other task, treating their mundane job as a sacred duty.

## 2. Dependencies
- Job System (`009-job-system`)
- Pop Factions / Beliefs (`068-pop-factions`, `197-civic-ideology`)
- Job Tenure / Hyper-specialization (`264-hyper-specialized-evolution`)

## 3. RED Phase: Tests First

```rust
// tests/layer1/social/mundane_cult_tests.rs

#[test]
fn test_long_tenure_in_menial_job_spawns_cult_faction() {
    // Arrange: Create several Pops performing a low-tier job (e.g., Hauling) for an extended duration.
    // Act: Trigger the faction/cult evaluation system.
    // Assert: A new Faction is created of type "MundaneCult" focused on the specific JobType.
}

#[test]
fn test_cult_members_refuse_job_reassignment() {
    // Arrange: A Pop belonging to a MundaneCult (e.g., "Cult of the Mop").
    // Act: Player attempts to assign the Pop to a different job (e.g., Science).
    // Assert: Assignment fails or the Pop immediately abandons the new job, suffering a massive mood penalty for "Forsaking the Sacred Duty".
}

#[test]
fn test_cult_members_gain_massive_efficiency_in_sacred_job() {
    // Arrange: A Pop belonging to a MundaneCult performing their sacred job.
    // Act: Calculate job efficiency/speed.
    // Assert: Efficiency is significantly boosted (e.g., +50% speed).
}

#[test]
fn test_cult_demands_shrine_construction() {
    // Arrange: An active MundaneCult.
    // Act: Advance simulation to trigger faction demands.
    // Assert: The cult emits a `FactionDemand` for a specific Shrine building related to their job.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/mundane_cult.rs

use bevy_ecs::prelude::*;
// Implement MundaneCult faction logic, job refusal logic, and efficiency modifiers.
```

## 5. REFACTOR Phase: Quality & Design
- Hook into the `JobTenure` component introduced in Spec 264 (`Hyper-Specialized Evolution`). `MundaneCult` is essentially a social/faction evolution of `JobTenure` rather than a biological one.
- Tie the job refusal logic into the `UtilityAI` scoring system. If a job does not match the cult's sacred job, the score should be heavily penalized, effectively preventing selection.
- Create a generic `Shrine` building type that can be parameterized with the cult's focus.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] Test coverage ≥85% for `mundane_cult.rs`.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Cults form organically from extended menial labor.
- [ ] Cultists refuse non-sacred jobs.
- [ ] Attempting to automate their sacred job (e.g., with drones) causes unrest among cultists.

## 7. Technical Guidance
- Define what constitutes a "menial" or "low-tier" job. Add a flag or enum to `JobType` (e.g., `tier: JobTier::Menial`).
- Use the existing `Faction` system to represent the cult. Give the faction a specific `Ideology` or trait indicating it is a Mundane Cult.
- When generating the name of the cult, use the `JobType` string representation (e.g., "The Holy Order of Haulers").

## 8. Questions
*Builder: add questions here if spec is unclear.*
