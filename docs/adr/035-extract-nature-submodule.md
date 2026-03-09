# 35. Extract Nature Sub-module

Date: 2024-05-28

## Status

Accepted

## Context

The `src/layer1/mod.rs` module had grown into a 220+ file monolithic module. It contained environment, physics, weather, and basic survival systems deeply coupled with core game logic. This violated domain boundaries and made the top-level Layer 1 namespace overwhelming and difficult to navigate.

## Decision

Extracted 13 foundational logic modules (`terrain`, `water`, `weather`, `atmosphere`, `temperature`, `seasons`, `wind`, `erosion`, `fertility`, `solar`, `ecology`, `radioactive`, `fire`) into a new `src/layer1/nature` module.

## Consequences

### Positive
*   **Domain Boundaries**: Enforces a stronger domain boundary around environmental physics, separating them from core entity and economy logic.
*   **Namespace Clarity**: Simplifies the top-level `layer1` namespace, making it easier for developers to find relevant systems.
*   **Maintainability**: Groups related environmental systems together, improving code organization and readability.

### Negative
*   **Refactoring Cost**: Required moving files and updating numerous imports across the codebase.
