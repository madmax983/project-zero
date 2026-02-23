# 27. Job System Simplification

Date: 2024-05-25

## Status

Accepted

## Context

In earlier iterations of the simulation, the colony used a granular "Job System" where each Pop was assigned a specific role via `AssignmentType`. Roles included `Miner`, `Doctor`, `Builder`, `Hauler`, etc. This approach led to several issues:

1.  **Assignment Rigidity**: A pop assigned as a `Miner` would idle if there were no mining designations, even if critical construction or hauling tasks were pending.
2.  **Micro-Management**: The player (or the automated `JobBoard`) had to constantly re-assign pops as needs shifted tick-by-tick.
3.  **Complexity Explosion**: Each new mechanic (e.g., Medicine) required a new `AssignmentType`, a new `WorkAI` state, and specific UI handling.
4.  **Flavor Disconnect**: Pops felt like "units" rather than individuals making choices based on needs.

This created a situation often referred to as "Assignment Hell," where the simulation struggled to balance workforce allocation effectively.

## Decision

We have decided to **simplify the Job System** by removing all task-specific assignments in favor of a hybrid approach:

1.  **Location-Bound Assignments Only**: We retain `AssignmentType` only for roles that require long-term commitment to a specific building or location.
    *   **Retained**: `FarmWorker` (tied to a Farm), `LibraryWorker` (tied to a Library), `Administrator` (tied to an Office), `ObservatoryWorker` (tied to an Observatory).
    *   **Removed**: `Miner`, `Builder`, `Hauler`, `Doctor`, `Refiner`.

2.  **Ad-Hoc Utility Actions**: All general labor is now handled dynamically by the **Utility AI** via `ActionType`.
    *   Any pop without a critical need or a specific location-bound assignment can evaluate and perform `ActionType::Mine`, `ActionType::Build`, `ActionType::Haul`, etc., based on current utility scores.
    *   Prioritization is handled by the scoring system (e.g., `urgency` of construction vs. `need` for resources), not by rigid job slots.

3.  **System-Driven Services**: Some roles, like `Doctor`, were removed entirely. Healing is now handled by the `MedicalSystem` operating on `Hospital` buildings and `Patient` assignments, rather than requiring an active `Doctor` pop to perform a "Heal" action.

## Consequences

### Positive
*   **Flexibility**: The colony adapts organically to changing needs. If a sudden construction project appears, available pops swarm to it without manual reassignment.
*   **Code Simplification**: Removed complex job-switching logic and specific AI states for `Doctor` or `Miner`. The `UtilityAI` provides a unified framework for all behavior.
*   **Reduced Idle Time**: Pops are rarely "stuck" in a role with no work.

### Negative
*   **Loss of Specialization**: Without specific "Jobs," pops do not (yet) gain skill proficiency as effectively in a single area, as they might switch between mining and farming frequently.
*   **Flavor Impact**: The UI no longer displays "Miner John" or "Doctor Jane," reducing the explicit narrative role of some pops.
*   **Tuning Sensitivity**: The system relies heavily on the Utility AI scoring functions being perfectly balanced. If `ActionType::Haul` is scored too high, no one will mine. If too low, industry stalls.
