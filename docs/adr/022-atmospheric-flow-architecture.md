# ADR 022: Atmospheric & Ventilation Flow Architecture

## Status
Accepted

## Context
The colony simulation requires dynamic fluid systems to model environmental hazards and life support:
1.  **Atmospheric Pollution**: Industrial buildings emit pollution which diffuses across the map, affecting Pop health.
2.  **Pressure**: Hull breaches cause rapid decompression, requiring a simulation of air pressure flow.

These systems interact with the built environment in complex ways:
*   **Walls** block flow completely.
*   **Vents** allow flow but block entity movement (except for specific entities like Vermin).
*   **Open Gates** allow both flow and movement.
*   **Airlocks** block flow but allow movement when cycled.

We needed a performant way to simulate these fluids without duplicating blocking logic across multiple grid systems (`AtmosphereGrid`, `PressureGrid`) or tightly coupling the simulation to the `Building` component.

## Decision

We decided to implement a **Double-Buffered Grid Architecture** with **Centralized Flow Transmissivity**.

### 1. Double-Buffered Grids
Both `AtmosphereGrid` and `PressureGrid` use a double-buffering strategy:
*   `values`: The current state of the simulation (read-only during update).
*   `scratch`: A secondary buffer for calculating the next state (write-only during update).

This approach allows for safe parallel updates (if needed in the future) and eliminates per-tick allocation churn by reusing the `scratch` vector.

### 2. Centralized Flow Transmissivity
We centralized the blocking logic in the `BuildingType` enum via the `flow_transmissivity()` method.
*   Returns `Option<f32>`:
    *   `None`: Full flow (1.0), transparent to the simulation (e.g., Farm, Stockpile).
    *   `Some(0.0)`: Full block (e.g., Wall, Airlock).
    *   `Some(1.0)`: Full flow but physically solid (e.g., Vent).
    *   `Some(0.5)`: Partial flow (e.g., Open Gate).

This serves as the single source of truth for all fluid simulations, preventing desync between Atmosphere and Pressure logic.

### 3. Capability-Based Pathfinding
To support the "Vent" mechanic (allows air but blocks Pops), we enhanced the pathfinding module:
*   **Vents** are marked as obstacles in `BuildingType::is_obstacle()`.
*   The `pathfinding` module accepts a capability flag (`can_use_vents`).
*   Entities like `Vermin` use this flag to traverse Vents, while standard Pops are blocked.

## Consequences

### Positive
*   **Performance**: Double buffering and flat `Vec<f32>` arrays are cache-friendly and allocation-free during the hot loop.
*   **Consistency**: `AtmosphereGrid` and `PressureGrid` share the exact same blocking rules via `BuildingType`.
*   **Extensibility**: Adding a new building that affects flow (e.g., a Filter) only requires updating `BuildingType::flow_transmissivity`.
*   ** decoupling**: The simulation systems (`update_atmosphere_system`) do not need to know about specific building types (like "Wall" or "Door"), only their transmissivity properties.

### Negative
*   **Coupling**: `BuildingType` becomes a central dependency for physics and simulation. Changing it triggers recompilation of all simulation modules.
*   **Complexity**: Double buffering requires careful memory management (`std::mem::swap`) and increases memory usage (2x grid size).

## Related
*   [ADR 001: Layered Architecture](./001-layered-architecture.md) - Defines the separation of Layer 1 simulation.
*   [ADR 020: Spontaneous Architecture](./020-spontaneous-architecture.md) - Defines dynamic building placement.
