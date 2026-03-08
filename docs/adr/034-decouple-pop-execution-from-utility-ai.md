# 34. Decouple Pop/Execution from UtilityAI

Date: 2026-02-17

## Status

Accepted

## Context

The `pop.rs` and `execution.rs` modules imported types such as `PopAction` and `UtilityWeights` from `utility_ai.rs`. This created a conceptual dependency from entity definitions and execution logic back to the AI system orchestrator itself. Although `utility_ai.rs` merely re-exported these types from `utility_types.rs`, the import path implied a deeper, tightly coupled architecture. This coupling violated a clean dependency graph and could lead to circular dependencies as the utility AI logic expanded.

## Decision

We updated the imports in `pop.rs`, `execution.rs`, and their associated tests to reference `utility_types.rs` directly instead of going through `utility_ai.rs`. This enforces a cleaner Directed Acyclic Graph (DAG) where Entity definitions (`pop`) and Execution logic (`execution`) depend only on shared Types (`utility_types`), and not on the AI System itself (`utility_ai`).

## Consequences

### Positive
*   **Clearer Architecture**: Enforces a strict one-way dependency flow, making the DAG easier to reason about.
*   **Reduced Coupling**: Loosens the connection between core entity logic and complex AI orchestration.
*   **Prevented Circular Dependencies**: Protects against future architectural entanglement.
*   **Test Stability**: Fixed failing integration tests (`hydroponics`, `lighting`, `funeral`) that were inadvertently coupled due to uninitialized `TechState` capacity and population interference.

### Negative
*   **None**: This is a direct structural improvement to internal module dependencies with no detrimental effects on the running simulation.
