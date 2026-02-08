# 8. Modular Utility AI Structure

Date: 2024-06-03

## Status

Accepted

## Context

As the number of behaviors in the simulation grew, the `utility_ai.rs` module became a monolithic "God Class." It contained the evaluation logic for every possible action (Eating, Sleeping, Working, Hauling, Researching, etc.), leading to several issues:
1.  **Maintainability:** The file was becoming too large to navigate easily.
2.  **Coupling:** Changes to one action's logic could inadvertently affect others due to shared scope.
3.  **Testing Difficulty:** Unit testing individual action evaluations required setting up the entire Utility AI context.
4.  **Extensibility:** Adding a new action required modifying the core loop file, increasing the risk of merge conflicts.

We needed a structure that allows for *encapsulated* behavior logic while maintaining a central orchestration point for decision-making.

## Decision

We have refactored the Utility AI system into a **Modular Action Architecture**.

1.  **Action Modules:** Evaluation logic for specific actions is extracted into dedicated modules.
    *   **Generic Actions:** Placed in `src/layer1/actions/` (e.g., `work.rs`, `haul.rs`, `idle.rs`).
    *   **Domain-Specific Actions:** Placed in their respective domain modules (e.g., `evaluate_seek_medical_care` in `medical.rs`, `evaluate_bury_corpse` in `funeral.rs`).
2.  **Orchestrator Pattern:** The `utility_ai.rs` module now acts as an orchestrator. It imports the `evaluate_*` functions and calls them in the `evaluate_actions_system` loop.
3.  **Standard Signature:** All evaluation functions follow a consistent pattern, accepting necessary context (Needs, Weights, World State Query) and returning an `Option<(UtilityScore, Target)>`.

## Consequences

### Positive
*   **Separation of Concerns:** Each action module focuses solely on its specific logic.
*   **Parallel Development:** Multiple developers can work on different behaviors simultaneously without conflict.
*   **Testability:** Individual `evaluate_*` functions can be unit-tested in isolation (as seen in `utility_ai_work_tests.rs`).
*   **Scalability:** New actions can be added by creating a new file and adding a single line to the orchestrator.

### Negative
*   **Boilerplate:** The orchestrator must import and call each function manually. There is no dynamic "plugin" system (yet).
*   **Discovery:** Developers must know where to look for specific actions (actions folder vs domain folder).
*   **Performance:** Slightly increased function call overhead (negligible compared to the logic within).
