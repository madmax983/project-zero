# 1. Layered Architecture for Simulation Scaling

Date: 2024-10-24
Status: Accepted

## Context

The SCALE project aims to simulate a civilization across multiple orders of magnitude:
1.  **Colony Scale:** Individual pops, buildings, and terrain tiles.
2.  **System Scale:** Logistics between planets, orbital mechanics.
3.  **Interstellar Scale:** High-level interactions between star systems.

Implementing all these scales in a monolithic structure would lead to:
-   **High Coupling:** Simulation logic for a single pop might accidentally depend on interstellar trade routes.
-   **Cognitive Overload:** Developers would need to understand the entire system to make changes to one part.
-   **Performance Issues:** Difficulty in optimizing specific simulation loops (e.g., ticking colonies slower than the system map).

## Decision

We will adopt a strict Layered Architecture, organizing the code into distinct modules corresponding to the simulation scale:

-   `src/layer1/`: **Planetary/Colony Scale**. Handles terrain, pops, buildings, needs, and local resources.
-   `src/layer2/`: **System Scale**. Handles orbital mechanics, interplanetary logistics, and system-wide resources.
-   `src/layer3/`: **Interstellar Scale**. Handles star clusters, long-range communications, and meta-game progression.
-   `src/shared/`: **Cross-Cutting Concerns**. Handles shared types (Time, Input, Logging), utility functions, and the core GameState.

Dependencies should flow downwards or be mediated by `shared`. Layer 1 should not directly depend on Layer 2's internal logic, but they may communicate via shared Resources or Events.

## Consequences

**Positive:**
-   **Separation of Concerns:** Each layer focuses on a specific problem domain.
-   **Testability:** Layers can be tested in isolation (mocking the inputs from other layers).
-   **Parallel Development:** Different agents/developers can work on the Colony Sim and Space Sim simultaneously with minimal conflict.

**Negative:**
-   **Boilerplate:** Data often needs to be passed explicitly between layers or via resources.
-   **Shared Dependency Management:** The `shared` module is a potential bottleneck and "God Object" risk. It must be kept lean, containing only truly common types.
